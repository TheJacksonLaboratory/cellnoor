import { type Account, betterAuth } from "better-auth";
import { readConfig } from "./config";
import { getDbClient } from "./db";

const {
  publicAuthUrl,
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
  delete account.password;

  return account;
}

export const auth = betterAuth({
  baseURL: publicAuthUrl,
  secret: authSecret,
  database: await getDbClient(),
  user: {
    modelName: "person",
    fields: {
      emailVerified: "email_verified",
      createdAt: "created_at",
      updatedAt: "updated_at",
    },
    additionalFields: {
      institution_id: {
        type: "string",
      },
      is_staff: {
        type: "boolean",
      },
    },
  },
  session: {
    cookieCache: {
      enabled: true,
      strategy: "jwt",
      maxAge: 30 * 24 * 60 * 60, // 30 days
      refreshCache: true,
    },
    storeSessionInDatabase: false,
  },
  account: {
    fields: {
      userId: "person_id",
      providerId: "auth_provider",
      accountId: "auth_provider_user_id",
      createdAt: "created_at",
      updatedAt: "updated_at",
    },
    storeStateStrategy: "cookie",
    accountLinking: { trustedProviders: ["microsoft"] },
  },
  databaseHooks: {
    account: {
      create: {
        // @ts-expect-error we're manually deleting fields we don't need
        before: deleteUnnecessaryAccountFields,
      },
      update: {
        // @ts-expect-error we're manually deleting fields we don't need
        before: deleteUnnecessaryAccountFields,
      },
    },
  },
  socialProviders: {
    microsoft: {
      disableIdTokenSignIn: true,
      profilePhotoSize: 48,
      tenantId: microsoftEntraTenantId,
      clientId: microsoftEntraClientId,
      clientSecret: microsoftEntraClientSecret,
      overrideUserInfoOnSignIn: true,
      async mapProfileToUser({ tid }) {
        const dbClient = await getDbClient();

        const { rows: [{ institution_id }] } = await dbClient.query(
          `select institution.id as institution_id from institution where institution.microsoft_entra_tenant_id = $1::uuid`,
          [tid],
        );

        return { institution_id };
      },
    },
  },
  advanced: {
    cookiePrefix: "cellnoor-auth",
    database: { generateId: "uuid" },
  },
});
