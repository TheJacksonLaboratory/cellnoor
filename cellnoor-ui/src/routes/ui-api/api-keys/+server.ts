import type { RequestHandler } from "./$types";
import { apiKeyFromCookies } from "$lib/server/auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "$lib/server/auth/crypto";
import { readConfig } from "$lib/server/config";
import { getDbClient } from "$lib/server/db-client";
import { EncryptedApiKey } from "$lib/server/auth/api-key";
import { getUserByApiKeyFromDb, insertApiKeyIntoDb } from "$lib/server/auth/db";

function permissionDenied(): Response {
  return new Response(JSON.stringify({ error: "permission denied" }), {
    status: 401,
    "headers": { "Content-Type": "application/json" },
  });
}

export const POST: RequestHandler = async ({ cookies }) => {
  const currentApiKey = await apiKeyFromCookies(
    cookies,
    API_KEY_ENCRYPTION_SECRET,
  );
  if (!currentApiKey) {
    return permissionDenied();
  }

  const dbClient = await getDbClient();
  const userId = await getUserByApiKeyFromDb(currentApiKey, dbClient);
  const config = await readConfig();
  const newUnencryptedApiKey = EncryptedApiKey.newUnencrypted();
  const newEncryptedApiKey = await EncryptedApiKey.fromRandomValues(
    newUnencryptedApiKey,
    API_KEY_ENCRYPTION_SECRET,
    config.apiKeyPrefixLength,
  );

  const created_at = await insertApiKeyIntoDb(
    newEncryptedApiKey,
    userId,
    dbClient,
  );

  return new Response(
    JSON.stringify({ api_key: newUnencryptedApiKey.toHex(), created_at }),
    { headers: { "Content-Type": "application/json" } },
  );
};

export const DELETE: RequestHandler = async ({ cookies, request }) => {
  const currentApiKey = await apiKeyFromCookies(
    cookies,
    API_KEY_ENCRYPTION_SECRET,
  );
  if (!currentApiKey) {
    return permissionDenied();
  }

  const { apiKeyPrefix } = await request.json();
  if (!apiKeyPrefix) {
    return new Response(
      JSON.stringify({ error: "must supply an API key prefix to delete" }),
      { status: 422, headers: { "Content-Type": "application/json" } },
    );
  }

  const dbClient = await getDbClient();
  const userId = await getUserByApiKeyFromDb(currentApiKey, dbClient);
  const config = await readConfig();
  const currentApiKeyPrefix = currentApiKey.slice(0, config.apiKeyPrefixLength);
  await dbClient`delete from api_keys where user_id = ${userId} and prefix=decode(${apiKeyPrefix}, 'hex') and prefix != ${currentApiKeyPrefix}`;

  return new Response(JSON.stringify({}), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
};
