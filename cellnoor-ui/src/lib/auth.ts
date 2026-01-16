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
      is_admin: { type: "boolean" },
      is_biology_staff: { type: "boolean" },
      is_computational_staff: { type: "boolean" },
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
      adapter: {
        createJwk: async (webKey) => {
          const dbClient = await getDbClient();

          const data = { created_at: webKey.createdAt, expires_at: webKey.expiresAt, public_key: webKey.publicKey, private_key: webKey.privateKey };
          const result = await dbClient`insert into json_web_keys ${dbClient(data)} returning id`;
          const id = result[0].id;

          return {id, ...webKey};
        },
        getJwks: async () => {
          const dbClient = await getDbClient();
          const results = await dbClient`select id, public_key as publicKey, private_key as privateKey, created_at as createdAt, expires_at as expiresAt from json_web_keys`;

          return results;
        }
      },
      jwks: {
        // A signed JWT is valid for 180 days
        rotationInterval: 60 * 60 * 24 * 90, // 90 days
        gracePeriod: 60 * 60 * 24 * 90, // 90 days
      },
      jwt: {
        getSubject: (session) => {
          return session.user.userId;
        },
        audience: await readConfig().then((c) => c.apiUrl),
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
