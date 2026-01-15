import { betterAuth } from "better-auth";
import { sveltekitCookies } from "better-auth/svelte-kit";
import { getRequestEvent } from "$app/server";
import { readConfig, readSecrets } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import type { MicrosoftEntraIDProfile } from "better-auth/social-providers";
import { upsertPersonIntoDb } from "$lib/server/auth/db";
import { createAuthMiddleware, jwt } from "better-auth/plugins";

export const auth = betterAuth({
  baseURL: (await readConfig()).publicUrl,
  secret: (await readSecrets()).authSecret,
  user: {
    additionalFields: {
      userId: { type: "string" },
      is_admin: { type: "string" },
      is_biology_staff: { type: "string" },
      is_computational_staff: { type: "string" },
    },
  },
  session: {
    cookieCache: {
      strategy: "jwt",
      maxAge: 30 * 60, // 30 minutes
    },
  },
  socialProviders: {
    microsoft: {
      clientId: (await readSecrets()).microsoftEntraClientId,
      clientSecret: (await readSecrets()).microsoftEntraClientSecret,
      tenantId: (await readSecrets()).microsoftEntraTenant,
      mapProfileToUser: async (profile) => {
        const dbClient = await getDbClient();
        const { id, is_admin, is_biology_staff, is_computational_staff } =
          await upsertPersonIntoDb(profile, dbClient);

        return {
          userId: id,
          is_admin,
          is_biology_staff,
          is_computational_staff,
        };
      },
    },
  },
  plugins: [
    jwt({
      jwt: {
        getSubject: (session) => {
          return session.user.userId;
        },
        expirationTime: "30 minutes",
        definePayload(
          { user: { is_admin, is_biology_staff, is_computational_staff } },
        ) {
          return {
            is_admin,
            is_biology_staff,
            is_computational_staff,
          };
        },
      },
    }),
    sveltekitCookies(getRequestEvent),
  ],
  advanced: {
    cookiePrefix: "cellnoor-ui",
    database: {
      generateId: false,
    },
  },
});
