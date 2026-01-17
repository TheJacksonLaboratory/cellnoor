import { betterAuth, hostname } from "better-auth";
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

          const data = {
            created_at: webKey.createdAt,
            expires_at: webKey.expiresAt,
            public_key: webKey.publicKey,
            private_key: webKey.privateKey,
          };
          const result = await dbClient`insert into json_web_keys ${
            dbClient(data)
          } returning id`;
          const id: string = result[0].id;

          return { id, ...webKey };
        },
        getJwks: async () => {
          const dbClient = await getDbClient();
          const results: {
            id: string;
            created_at: Date;
            expires_at: Date;
            public_key: string;
            private_key: string;
          }[] = await dbClient`select * from json_web_keys`;

          return results.map(
            ({ id, created_at, expires_at, public_key, private_key }) => {
              return {
                id,
                createdAt: created_at,
                expiresAt: expires_at,
                publicKey: public_key,
                privateKey: private_key,
              };
            },
          );
        },
      },
      jwks: {
        rotationInterval: 60 * 60 * 24 * 90, // 180 days
        gracePeriod: 0,
      },
      jwt: {
        getSubject: (session) => {
          return session.user.userId;
        },
        issuer: await readConfig().then((c) => c.publicUrl) ??
          "http://localhost:5173", // Just assume that if `publicUrl` isn't set, we're serving on dev
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
