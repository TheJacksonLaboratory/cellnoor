import { apiKeyFromCookies } from "$lib/server/auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "$lib/server/auth/crypto";
import { getUserByApiKeyFromDb } from "$lib/server/auth/db";
import { getDbClient } from "$lib/server/db-client";
import { ApiClient } from "$lib/server/scamplers-client";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async (event) => {
  const apiClient = await ApiClient.new();

  // It would be slightly (but noticeably) faster to just grab the web summary from the database and send it directly. However, that requires us to do `set local role = ${userID}` to take advantage of Postgres's row-level security. Unfortunately, that requires the `scamplers-ui` database user to be granted all user roles. That's probably not a security issue, but it's probably better to give the UI as little privilege as possible because I don't know what I'm doing
  return await apiClient.getRaw(event);
};
