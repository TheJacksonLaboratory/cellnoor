import { betterAuth } from "better-auth";
import { sveltekitCookies } from "better-auth/svelte-kit";
import { getRequestEvent } from "$app/server";
import { createAuthMiddleware } from "better-auth/api";
import { readConfig, readSecrets } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import {
  API_KEY_ENCRYPTION_SECRET,
  AUTH_SECRET,
} from "$lib/server/auth/crypto";
import type { MicrosoftEntraIDProfile } from "better-auth/social-providers";
import { CookieNames } from "$lib/server/auth/cookies";
import { deleteApiKey, insertApiKey, insertPerson } from "$lib/server/auth/db";
import { EncryptedApiKey } from "$lib/server/auth/api-key";

const ONE_YEAR = 60 * 60 * 24 * 365;

const COOKIE_NAMES = [
  CookieNames.encryptedApiKey,
  CookieNames.apiKeyInitializationVector,
];

const COOKIE_OPTIONS = {
  secure: true,
  httpOnly: true,
  path: "/",
};

let microsoftEntraProfiles: Record<string, MicrosoftEntraIDProfile> = {};

export const auth = betterAuth({
  baseURL: (await readConfig()).publicUrl,
  secret: AUTH_SECRET,
  socialProviders: {
    microsoft: {
      clientId: (await readSecrets()).microsoft_entra_client_id,
      clientSecret: (await readSecrets()).microsoft_entra_client_secret,
      tenantId: (await readSecrets()).microsoft_entra_tenant,
      // This is a bit of a hack. We need the user's Microsoft Entra OID and tenant ID, which is only available in this function.
      mapProfileToUser: async (profile) => {
        // Using a user's email address as a unique key is typically poor practice because of one of the following (https://learn.microsoft.com/en-us/entra/identity-platform/id-token-claims-reference#payload-claims):
        // 1. an email address can be reassigned to a different person
        // 2. a user could sign in from two different browsers at the same time (same email address, different sessions)
        // It is extremely unlikely that either of these will cause a problem because this key-value pair exists for an infinitesimal period of time.
        microsoftEntraProfiles[profile.email] = profile;
      },
    },
  },
  session: {
    cookieCache: {
      enabled: true,
      strategy: "jwe",
      refreshCache: true,
      maxAge: ONE_YEAR,
    },
  },
  plugins: [sveltekitCookies(getRequestEvent)],
  hooks: {
    after: createAuthMiddleware(async (ctx) => {
      // Small helper function that returns the deleted cookie
      const deleteCookie = (cookieName: string) => {
        const cookie = ctx.getCookie(cookieName);
        ctx.setCookie(cookieName, "", { maxAge: 0 });

        return cookie;
      };

      const appConfig = await readConfig();
      const dbClient = await getDbClient();

      // If the user is signing out, erase cookies and delete the API key from the database. Note this check is necessarily performed before checking `newSession`
      if (ctx.path.includes("sign-out")) {
        const [encryptedApiKey, initializationVector] = COOKIE_NAMES.map(
          deleteCookie,
        );

        // Delete the API key from the database
        await deleteApiKey(
          {
            encryptedApiKey: encryptedApiKey!,
            initializationVector: initializationVector!,
            encryptionSecret: API_KEY_ENCRYPTION_SECRET,
            apiKeyPrefixLength: appConfig.apiKeyPrefixLength,
          },
          dbClient,
        );
      }

      const { newSession } = ctx.context;
      if (!newSession) {
        return;
      }

      const { email, emailVerified } = newSession.user;
      const microsoftEntraProfile = microsoftEntraProfiles[email];
      if (!microsoftEntraProfile) {
        return;
      }
      delete microsoftEntraProfiles[email];

      // Upsert the user in the database
      const personId = await insertPerson(
        {
          emailVerified,
          ...microsoftEntraProfile,
        },
        dbClient,
      );

      // Generate an encrypted API key
      const encryptedApiKey = await EncryptedApiKey.new(
        API_KEY_ENCRYPTION_SECRET,
        appConfig.apiKeyPrefixLength,
      );

      await insertApiKey(encryptedApiKey, personId, dbClient);

      const cookieValues = [
        encryptedApiKey.hexEncode(),
        encryptedApiKey.hexEncodedInitializationVector(),
      ];
      const setCookie = (cookieName: string, cookieValueIdx: number) => {
        ctx.setCookie(cookieName, cookieValues[cookieValueIdx]!, {
          maxAge: ONE_YEAR,
          ...COOKIE_OPTIONS,
        });
      };

      COOKIE_NAMES.map(setCookie);
    }),
  },
});
