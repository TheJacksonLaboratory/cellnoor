import { betterAuth } from "better-auth";
import { sveltekitCookies } from "better-auth/svelte-kit";
import { getRequestEvent } from "$app/server";
import { createAuthMiddleware } from "better-auth/api";
import { readConfig, readSecrets } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import type { MicrosoftEntraIDProfile } from "better-auth/social-providers";
import {

  upsertPersonIntoDb as upsertPersonIntoDb,
} from "$lib/server/auth/db";

let microsoftEntraProfiles: Record<string, MicrosoftEntraIDProfile> = {};

export const auth = betterAuth({
  baseURL: (await readConfig()).publicUrl,
  secret: (await readSecrets()).authSecret,
  socialProviders: {
    microsoft: {
      clientId: (await readSecrets()).microsoft_entra_client_id,
      clientSecret: (await readSecrets()).microsoft_entra_client_secret,
      tenantId: (await readSecrets()).microsoft_entra_tenant,
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
  session: {
    cookieCache: {
      strategy: "jwt",
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

      await upsertPersonIntoDb(
        {
          emailVerified,
          ...microsoftEntraProfile,
        },
        dbClient,
      );
    }),
  },
});
