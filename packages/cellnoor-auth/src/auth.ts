import { type Account, betterAuth } from "better-auth";
import { readConfig } from "./config";
import { getDbClient } from "./db";

const {
  publicAuthUrl,
  publicApiUrl,
  publicUiUrl,
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

async function provisionDbUser({ id }: { id: string }) {
  const dbClient = await getDbClient();
  await dbClient.query(
    "select create_person_user_if_not_exists($1)",
    [id],
  );
}


export const auth = betterAuth({
  baseURL: publicAuthUrl,
  secret: authSecret,
  trustedOrigins: [publicApiUrl, publicUiUrl],
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
      institution_id: {
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
    storeSessionInDatabase: false,
  },
  account: {
    modelName: "person_account",
    fields: {
      userId: "person_id",
      providerId: "auth_provider_name",
      accountId: "auth_provider_user_id",
      createdAt: "created_at",
      updatedAt: "updated_at",
    },
    storeStateStrategy: "cookie",
    storeAccountCookie: true,
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
    user: {
      update: {
        after: provisionDbUser
      },
      create: {
        after: provisionDbUser
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
        const { rows: [{ institution_id }] } = await dbClient.query(
          "select id as institution_id from institution where microsoft_entra_tenant_id = $1::uuid",
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
