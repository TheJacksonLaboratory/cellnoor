import { betterAuth } from "better-auth";
import { sveltekitCookies } from "better-auth/svelte-kit";
import { getRequestEvent } from "$app/server";
import { readConfig, readSecrets } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import { getUserProjects, getUserRoles, upsertPersonIntoDb } from "$lib/server/auth/db";
import { jwt } from "better-auth/plugins";
import { createAuthMiddleware } from "better-auth/api";
import { API_TOKEN_COOKIE_NAME } from "./server/cellnoor-client";

export const auth = betterAuth({
  baseURL: await readConfig().then(({ publicUrl }) => publicUrl),
  secret: await readSecrets().then(({ authSecret }) => authSecret),
  user: {
    additionalFields: {
      user_id: { type: "string" },
      is_admin: { type: "boolean" },
      is_biology_staff: { type: "boolean" },
      is_computational_staff: { type: "boolean" },
    },
  },
  session: {
    cookieCache: {
      strategy: "jwt",
      maxAge: 7 * 24 * 60 * 60, // 1 week
    },
  },
  socialProviders: {
    microsoft: {
      clientId: await readSecrets().then(({ microsoftEntraClientId }) => microsoftEntraClientId),
      clientSecret: await readSecrets().then(
        ({ microsoftEntraClientSecret }) => microsoftEntraClientSecret,
      ),
      tenantId: await readSecrets().then(({ microsoftEntraTenant }) => microsoftEntraTenant),
      mapProfileToUser: async (profile) => {
        // It's useful to have the user's roles in the JWT assigned by better-auth in order to display certain UI
        // elements according to the user's roles. However, it's not great that this function only runs on a fresh
        // sign-in (the user authenticates with Microsoft) because that means they have to sign out and sign in again
        // to see changes reflected in the UI. However, in practice, I don't think this will happen often enough that
        // it's a problem.
        const dbClient = await getDbClient();
        const { id, is_admin, is_biology_staff, is_computational_staff } = await upsertPersonIntoDb(
          profile,
          dbClient,
        );

        return {
          id,
          user_id: id,
          is_admin,
          is_biology_staff,
          is_computational_staff,
        };
      },
    },
  },
  hooks: {
    after: createAuthMiddleware(async (resp) => {
      if (resp.path.includes("sign-out")) {
        resp.setCookie(API_TOKEN_COOKIE_NAME, "", { path: "/", maxAge: 0 });
      }
    }),
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
          const result = await dbClient`insert into json_web_keys ${dbClient(data)} returning id`;
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

          return results.map(({ id, created_at, expires_at, public_key, private_key }) => {
            return {
              id,
              createdAt: created_at,
              expiresAt: expires_at,
              publicKey: public_key,
              privateKey: private_key,
            };
          });
        },
      },
      jwks: {
        rotationInterval: 60 * 60 * 24 * 90, // 180 days
        gracePeriod: 0,
      },
      jwt: {
        getSubject: (session) => {
          return session.user.user_id;
        },
        issuer: await readConfig().then(({ jwtIssuer }) => jwtIssuer),
        audience: await readConfig().then(({ jwtAudience }) => jwtAudience),
        expirationTime: "1 hour",
        async definePayload({ user: { user_id } }) {
          const dbClient = await getDbClient();

          const { is_admin, is_biology_staff, is_computational_staff } = await getUserRoles(
            user_id,
            dbClient,
          );

          const isStaff = is_admin || is_biology_staff || is_computational_staff;

          const projects = isStaff
            ? { quantity: "all" }
            : await getUserProjects(user_id, dbClient).then(({ projects }) => {
                return { quantity: "some", project_ids: projects };
              });

          return {
            jti: Bun.randomUUIDv7(),
            private_claims: {
              user_id,
              is_admin,
              is_biology_staff,
              is_computational_staff,
            },
            projects,
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
