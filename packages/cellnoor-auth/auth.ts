import { type Account, betterAuth } from "better-auth";
import { readConfig } from "./config";
import { getDbClient } from "./db";

const {
  publicBaseUrl,
  authSecret,
  microsoftEntraCommonTenant,
  microsoftEntraClientId,
  microsoftEntraClientSecret,
} = await readConfig();

async function deleteUnnecessaryAccountFields(account: Account) {
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
  baseURL: publicBaseUrl,
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
  secondaryStorage: {
    get: () => null,
    getAndDelete: () => null,
    increment: () => 0,
    set: () => null,
    delete: () => null,
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
      disableProfilePhoto: true,
      tenantId: microsoftEntraCommonTenant ? "common" : "organizations",
      clientId: microsoftEntraClientId,
      clientSecret: microsoftEntraClientSecret,
      overrideUserInfoOnSignIn: true,
      async mapProfileToUser({ tid, oid, email }) {
        const dbClient = await getDbClient();

        const {
          rows: [{ institution_id }],
        } = await dbClient.query(
          `select institution.id as institution_id from institution where institution.microsoft_entra_tenant_id = $1::uuid`,
          [tid],
        );

        // When a person signs in, better-auth will find the row in the person table that has the same email (if it
        // exists) and link it to the new account. This could be catastrophic in the following scenario:
        // 1. Ahmed Said (ahmed.said@jax.org) is a highly privileged user (and he's super smart and cool). Call him AS1
        // 2. He leaves
        // 3. A new person named Ahmed Said (let's call him AS2) joins. Since AS1 left, the email ahmed.said@jax.org is
        // now available
        // 4. When AS2 signs in, better-auth will find the row in the person table that belonged to AS1 and link AS2's
        // account to it
        // 5. Now AS2 is highly privileged
        //
        // To guard against this, we remove the email from AS1 before better-auth completes the sign-in, so that AS2
        // gets a new row in the person table rather than getting AS1's row. This wouldn't be a problem if we didn't
        // default person.email_verified = true, but we have to do that because better-auth only links a sign-in to an
        // existing person if email_verified = true, and the main path to creating someone is not through sign-in, but
        // through creation by an automated script (https://github.com/demhadais/update-cellnoor).
        await dbClient.query(
          `update person set email = null
          where email = $1 and exists (
              select 1 from account
              where
                  account.person_id = person.id
                  and not (account.auth_provider = 'microsoft' and account.auth_provider_user_id = $2)
          )`,
          [email, oid],
        );

        return { institution_id };
      },
    },
  },
  disabledPaths: [
    "/sign-up/email",
    "/sign-in/email",
    "/reset-password",
    "/verify-password",
    "/verify-email",
    "/send-verification-email",
    "/change-email",
    "/change-password",
    "/update-session",
    "/update-user",
    "/delete-user",
    "/request-password-reset",
    "/list-sessions",
    "/revoke-session",
    "/revoke-sessions",
    "/revoke-other-sessions",
    "/link-social",
    "/list-accounts",
    "/delete-user/callback",
    "/unlink-account",
    "/refresh-token",
    "/get-access-token",
    "/account-info",
    "/ok",
  ],
  advanced: {
    cookiePrefix: "cellnoor-auth",
    database: { generateId: "uuid", validateSchema: false },
  },
  logger: { level: "error" },
});
