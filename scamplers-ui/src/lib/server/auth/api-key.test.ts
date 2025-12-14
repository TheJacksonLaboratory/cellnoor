import { EncryptedApiKey } from "./api-key";
import { decryptHexEncodedApiKey, ENCRYPTION_ALGORITHM } from "./crypto";
import { expect, test } from "bun:test";

test("API key should be the same after encryption and decryption", async () => {
  const usages: KeyUsage[] = ["decrypt", "encrypt"];
  const secret = await crypto.subtle.generateKey(
    ENCRYPTION_ALGORITHM,
    true,
    usages,
  );

  const randomValues = EncryptedApiKey.newUnencrypted();
  const encryptedApiKey = await EncryptedApiKey.fromRandomValues(
    randomValues,
    secret,
    8,
  );

  const decryptedApiKey = new Uint8Array(
    await decryptHexEncodedApiKey(
      encryptedApiKey.hexEncodedInitializationVector(),
      secret,
      encryptedApiKey.hexEncode(),
    ),
  );

  expect(decryptedApiKey).toEqual(randomValues);
});
