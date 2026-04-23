import { betterAuth } from "better-auth";
import { readConfig } from "./config";
import { getDbClient } from "./db";

const {
  publicUrl,
  authSecret,
  microsoftEntraTenantId,
  microsoftEntraClientId,
  microsoftEntraClientSecret,
} = await readConfig();

export const auth = betterAuth({
  baseURL: publicUrl,
  secret: authSecret,
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
        references: { model: "organization", field: "id" },
      },
    },
  },
  session: {
    cookieCache: {
      strategy: "jwt",
      maxAge: 7 * 24 * 60 * 60, // 1 week
      refreshCache: true,
    },
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
  socialProviders: {
    microsoft: {
      tenantId: microsoftEntraTenantId,
      clientId: microsoftEntraClientId,
      clientSecret: microsoftEntraClientSecret,
      async mapProfileToUser({ tid }) {
        const dbClient = await getDbClient();
        const { rows: [{ organization_id }] } = await dbClient.query(
          "select id from organization where microsoft_entra_tenant_id = $1",
          [tid],
        );

        return { organization_id };
      },
    },
  },
  database: await getDbClient(),
  advanced: {
    cookiePrefix: "cellnoor-auth",
    database: { generateId: "uuid" },
  },
});
