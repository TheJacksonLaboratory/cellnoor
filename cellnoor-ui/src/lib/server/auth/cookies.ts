import type { Cookies } from "@sveltejs/kit";
import { API_KEY_ENCRYPTION_SECRET, decryptHexEncodedApiKey } from "./crypto";
import { readConfig } from "../config";

export class CookieNames {
  static get encryptedApiKey(): string {
    return "cellnoor.encrypted_api_key";
  }
  static get apiKeyInitializationVector(): string {
    return "cellnoor.api_key_initialization_vector";
  }
}

export async function apiKeyFromCookies(
  cookies: Cookies,
  encryptionSecret: CryptoKey,
): Promise<ArrayBuffer | null> {
  const initializationVector = cookies.get(
    CookieNames.apiKeyInitializationVector,
  );
  const hexEncodedEncryptedApiKey = cookies.get(CookieNames.encryptedApiKey);

  if (!initializationVector || !hexEncodedEncryptedApiKey) {
    return null;
  }

  return await decryptHexEncodedApiKey(
    initializationVector,
    encryptionSecret,
    hexEncodedEncryptedApiKey,
  );
}
