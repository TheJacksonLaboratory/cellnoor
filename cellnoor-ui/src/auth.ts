import { betterAuth } from "better-auth";
import { sveltekitCookies } from "better-auth/svelte-kit";
import { getRequestEvent } from "$app/server";
import { createAuthMiddleware } from "better-auth/api";
import { readConfig, readSecrets } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import type { MicrosoftEntraIDProfile } from "better-auth/social-providers";
import { upsertPersonIntoDb } from "$lib/server/auth/db";
import jwt from "jsonwebtoken";

let microsoftEntraProfiles: Record<string, MicrosoftEntraIDProfile> = {};

// https://www.better-auth.com/docs/concepts/session-management#basic-stateless-setup
const SEVEN_DAYS = 7 * 24 * 60 * 60;

export const auth = betterAuth({
  baseURL: (await readConfig()).publicUrl,
  secret: (await readSecrets()).authSecret,
  socialProviders: {
    microsoft: {
      clientId: (await readSecrets()).microsoftEntraClientId,
      clientSecret: (await readSecrets()).microsoftEntraClientSecret,
      tenantId: (await readSecrets()).microsoftEntraTenant,
      // This is a bit of a hack. We need the user's Microsoft Entra OID and tenant ID, which is only available in this function.
      mapProfileToUser: async (profile) => {
        // Using a user's email address as a unique key is typically poor practice because of one of the following (https://learn.microsoft.com/en-us/entra/identity-platform/id-token-claims-reference#payload-claims):
        // 1. an email address can be reassigned to a different person
        // 2. a user could sign in from two different browsers at the same time (same email address, different sessions)
        // It is extremely unlikely that either of these will cause a problem because this key-value pair exists for an infinitesimal period of time.
        microsoftEntraProfiles[profile.email] = profile;
      },
    },
  },
  plugins: [sveltekitCookies(getRequestEvent)],
  hooks: {
    after: createAuthMiddleware(async (ctx) => {
      const dbClient = await getDbClient();

      const { newSession } = ctx.context;
      if (!newSession) {
        return;
      }

      const { email, emailVerified } = newSession.user;
      const microsoftEntraProfile = microsoftEntraProfiles[email];
      if (!microsoftEntraProfile) {
        return;
      }
      delete microsoftEntraProfiles[email];

      const userId = await upsertPersonIntoDb(
        {
          emailVerified,
          ...microsoftEntraProfile,
        },
        dbClient,
      );

      // In its infinite wisdome, better-auth sets some kind of user ID that you have no control over, so we make our
      // own JWT that the (beautiful) backend can parse and verify. We also use 7 days for the max-age because that's
      // better-auth's default.
      // (https://www.better-auth.com/docs/concepts/session-management#basic-stateless-setup)
      const signedJwt = jwt.sign({}, ctx.context.secret, { subject: userId, jwtid: Bun.randomUUIDv7(), expiresIn: "7d", algorithm: "HS512" })
      ctx.setCookie("cellnoor-ui.api_token", signedJwt, {
        sameSite: "strict",
        secure: true,
        httpOnly: true,
        path: "/"
      })
    }),
  },
  advanced: {
    cookiePrefix: "cellnoor-ui",

  }
});
