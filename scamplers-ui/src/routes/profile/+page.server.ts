import { EncryptedApiKey } from "$lib/server/auth/api-key";
import { userIdFromCookies } from "$lib/server/auth/cookies";
import { apiKeyFromCookies } from "$lib/server/auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "$lib/server/auth/crypto";
import { insertApiKey } from "$lib/server/auth/db";
import { readConfig } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import type { PageServerLoad } from "./$types.js";

export const load: PageServerLoad = async ({ cookies }) => {
  const config = await readConfig();
  const apiKeyPrefixLength = config.apiKeyPrefixLength;

  const thisSessionApiKey = await apiKeyFromCookies(
    cookies,
    API_KEY_ENCRYPTION_SECRET,
  );
  const thisSessionApiKeyPrefix = thisSessionApiKey?.slice(
    0,
    apiKeyPrefixLength,
  );

  const dbClient = await getDbClient();
  const userId = await userIdFromCookies(cookies, dbClient);
  const apiKeyPrefixResults: { prefix: string }[] =
    await dbClient`select encode(prefix, 'hex') as prefix from api_keys where user_id = ${userId} and prefix != ${thisSessionApiKeyPrefix} order by prefix`;

  return {
    apiKeyPrefixes: apiKeyPrefixResults.map(
      ({ prefix }) => {
        return prefix;
      },
    ),
  };
};

export const actions = {
  createApiKey: async ({ cookies }) => {
    const config = await readConfig();
    const apiKeyPrefixLength = config.apiKeyPrefixLength;
    const dbClient = await getDbClient();

    const newUnencryptedApiKey = EncryptedApiKey.newUnencrypted();
    const newEncryptedApiKey = await EncryptedApiKey.fromRandomValues(
      newUnencryptedApiKey,
      API_KEY_ENCRYPTION_SECRET,
      apiKeyPrefixLength,
    );
    const userId = await userIdFromCookies(cookies, dbClient) as string;

    await insertApiKey(newEncryptedApiKey, userId, dbClient);

    return newUnencryptedApiKey.toHex();
  },
  deleteApiKey: async ({ cookies, request }) => {
    const config = await readConfig();
    const apiKeyPrefixLength = config.apiKeyPrefixLength;
    const dbClient = await getDbClient();

    const userId = await userIdFromCookies(cookies, dbClient);

    const thisSessionApiKey = await apiKeyFromCookies(
      cookies,
      API_KEY_ENCRYPTION_SECRET,
    );
    const thisSessionApiKeyPrefix = thisSessionApiKey?.slice(
      0,
      apiKeyPrefixLength,
    );

    const formData = await request.formData();

    await dbClient`delete from api_keys where user_id = ${userId} and prefix=decode(${
      formData.get("apiKeyPrefix")
    }, 'hex') and prefix != ${thisSessionApiKeyPrefix};`;
  },
};
