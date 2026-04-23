import { type Account, betterAuth, z } from "better-auth";
import { readConfig } from "./config";
import { getDbClient } from "./db";

const {
  publicAuthUrl,
  publicAppUrl,
  authSecret,
  microsoftEntraTenantId,
  microsoftEntraClientId,
  microsoftEntraClientSecret,
} = await readConfig();

async function deleteUnnecessaryAccountFields(
  account: Account,
) {
  delete account.accessToken;
  delete account.refreshToken;
  delete account.accessTokenExpiresAt;
  delete account.refreshToken;
  delete account.idToken;
  delete account.refreshTokenExpiresAt;

  return account;
}

export const auth = betterAuth({
  baseURL: publicAuthUrl,
  secret: authSecret,
  trustedOrigins: [publicAppUrl],
  database: await getDbClient(),
  // We need to supply secondary storage to make better-auth forget about our database, so we just provide a dummy implementation
  secondaryStorage: {
    get: () => null,
    set: () => null,
    delete: () => null,
  },
  user: {
    modelName: "person",
    fields: {
      emailVerified: "email_verified",
      createdAt: "created_at",
      updatedAt: "updated_at",
    },
    additionalFields: {
      organization_id: {
        type: "string",
      },
    },
  },
  session: {
    cookieCache: {
      enabled: true,
      strategy: "jwt",
      maxAge: 7 * 24 * 60 * 60, // 1 week
      refreshCache: true,
    },
    // Despite setting the fact that you are telling better-auth to store sessions as JWTs in cookies AND to not store them in the database, you still have to provide a secondary storage
    storeSessionInDatabase: false,
  },
  account: {
    modelName: "person_account",
    fields: {
      userId: "person_id",
      providerId: "auth_provider_id",
      accountId: "auth_provider_user_id",
      createdAt: "created_at",
      updatedAt: "updated_at",
    },
    storeStateStrategy: "cookie",
    storeAccountCookie: true,
  },
  databaseHooks: {
    account: {
      create: {
        before: deleteUnnecessaryAccountFields,
      },
      update: {
        before: deleteUnnecessaryAccountFields,
      },
    },
  },
  socialProviders: {
    microsoft: {
      tenantId: microsoftEntraTenantId,
      clientId: microsoftEntraClientId,
      clientSecret: microsoftEntraClientSecret,
      async mapProfileToUser({ tid }) {
        const dbClient = await getDbClient();
        const { rows: [{ organization_id }] } = await dbClient.query(
          "select id as organization_id from organization where microsoft_entra_tenant_id = $1::uuid",
          [tid],
        );

        return { organization_id };
      },
    },
  },
  advanced: {
    cookiePrefix: "cellnoor-auth",
    database: { generateId: "uuid" },
  },
});
