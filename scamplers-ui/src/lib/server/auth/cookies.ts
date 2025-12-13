import type { Cookies } from "@sveltejs/kit";
import { API_KEY_ENCRYPTION_SECRET, decryptApiKey } from "./crypto";
import { readConfig } from "../config";

export class CookieNames {
  static get encryptedApiKey(): string {
    return "scamplers.encrypted_api_key";
  }
  static get apiKeyInitializationVector(): string {
    return "scamplers.api_key_initialization_vector";
  }
}

export async function apiKeyFromCookies(
  cookies: Cookies,
  encryptionSecret: CryptoKey,
) {
  const initializationVector = cookies.get(
    CookieNames.apiKeyInitializationVector,
  );
  const hexEncodedEncryptedApiKey = cookies.get(CookieNames.encryptedApiKey);

  if (!initializationVector || !hexEncodedEncryptedApiKey) {
    return null;
  }

  return await decryptApiKey(
    initializationVector,
    encryptionSecret,
    hexEncodedEncryptedApiKey,
  );
}

export async function hexEncodedApiKeyFromCookies(
  cookies: Cookies,
  encryptionSecret: CryptoKey,
): Promise<string | null> {
  const decryptedBytes = await apiKeyFromCookies(cookies, encryptionSecret);
  if (!decryptedBytes) {
    return null;
  }

  return new Uint8Array(decryptedBytes).toHex();
}

export async function userIdFromCookies(
  cookies: Cookies,
  dbClient: Bun.SQL,
): Promise<string | null> {
  const config = await readConfig();
  const apiKeyPrefixLength = config.apiKeyPrefixLength;

  const userApiKey = await apiKeyFromCookies(
    cookies,
    API_KEY_ENCRYPTION_SECRET,
  );
  if (!userApiKey) {
    return null;
  }

  const userApiKeyPrefix = userApiKey.slice(0, apiKeyPrefixLength);
  const results =
    await dbClient`select user_id from api_keys where prefix = ${userApiKeyPrefix};`;

  return results[0].user_id;
}
