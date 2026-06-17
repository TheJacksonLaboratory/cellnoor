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

  return account;
}

async function provisionDbUser({ id }: { id: string }) {
  const dbClient = await getDbClient();
  await dbClient.query(
    "select create_app_user_if_not_exists($1::uuid)",
    [id],
  );
}

function getDomain() {
  return publicAuthUrl.split(".").toSpliced(0,1).join(".")
}

export const auth = betterAuth({
  baseURL: publicAuthUrl,
  trustedOrigins: () => [`*.${getDomain()}`, getDomain()],
  secret: authSecret,
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
      is_staff: {
        type: "boolean"
      },
      can_manage_users: {
        type: "boolean"
      }
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
    fields: {
      userId: "person_id",
      providerId: "auth_provider",
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
      disableIdTokenSignIn: true,
      tenantId: microsoftEntraTenantId,
      clientId: microsoftEntraClientId,
      clientSecret: microsoftEntraClientSecret,
      overrideUserInfoOnSignIn: true,
      async mapProfileToUser({ tid, oid }) {
        const dbClient = await getDbClient();

        const { rows: [{ institution_id }] } = await dbClient.query(
          `select institution.id as institution_id from institution
          where institution.microsoft_entra_tenant_id = $1::uuid`,
          [tid],
        );

        return { institution_id, id: oid };
      },
    },
  },
  advanced: {
    cookiePrefix: "cellnoor-auth",
    crossSubDomainCookies: { enabled: true, domain: getDomain() },
    database: { generateId: "uuid" },
  },
});
