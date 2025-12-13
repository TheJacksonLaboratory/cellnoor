import type { RequestHandler } from "./$types";
import { apiKeyFromCookies, userIdFromCookies } from "$lib/server/auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "$lib/server/auth/crypto";
import { readConfig } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import { EncryptedApiKey } from "$lib/server/auth/api-key";
import { insertApiKey } from "$lib/server/auth/db";

export const GET: RequestHandler = async ({ cookies }) => {
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

  return new Response(
    JSON.stringify({
      apiKeyPrefixes: apiKeyPrefixResults.map(
        ({ prefix }) => {
          return prefix;
        },
      ),
    }),
    { headers: { "Content-Type": "application/json" } },
  );
};

export const POST: RequestHandler = async ({ cookies }) => {
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

  return new Response(
    JSON.stringify({ apiKey: newUnencryptedApiKey.toHex() }),
    { headers: { "Content-Type": "application/json" } },
  );
};

export const DELETE: RequestHandler = async ({ cookies, request }) => {
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

  const { apiKeyPrefix } = await request.json();

  await dbClient`delete from api_keys where user_id = ${userId} and prefix=decode(${apiKeyPrefix}, 'hex') and prefix != ${thisSessionApiKeyPrefix};`;

  return new Response();
};
