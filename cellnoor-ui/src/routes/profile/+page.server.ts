import { EncryptedApiKey } from "$lib/server/auth/api-key";
import { apiKeyFromCookies } from "$lib/server/auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "$lib/server/auth/crypto";
import { getUserByApiKeyFromDb, insertApiKeyIntoDb } from "$lib/server/auth/db";
import { readConfig } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import type { PageServerLoad } from "./$types.js";

export const load: PageServerLoad = async ({ cookies }) => {
  const apiKey = await apiKeyFromCookies(
    cookies,
    API_KEY_ENCRYPTION_SECRET,
  );

  const dbClient = await getDbClient();
  const userId = await getUserByApiKeyFromDb(apiKey!, dbClient);

  const config = await readConfig();
  const thisSessionApiKeyPrefix = apiKey?.slice(0, config.apiKeyPrefixLength);
  const apiKeyPrefixes: { prefix: string; created_at: Date }[] =
    await dbClient`select encode(prefix, 'hex') as prefix, created_at from api_keys where user_id = ${userId} and prefix != ${thisSessionApiKeyPrefix} order by created_at`;

  return {
    apiKeyPrefixes,
    apiKeyPrefixLength: config.apiKeyPrefixLength,
  };
};
