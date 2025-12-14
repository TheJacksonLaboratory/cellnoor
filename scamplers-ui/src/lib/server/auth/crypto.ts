export const ENCRYPTION_ALGORITHM = {
  name: "AES-GCM",
  length: 256,
};

async function getEncryptionSecret(filePath: string): Promise<CryptoKey> {
  const file = Bun.file(filePath);
  const usages: KeyUsage[] = ["decrypt", "encrypt"];

  const fileExists = await file.exists();

  // If the file has been created, import the key from there. If not, create one.
  const secret = fileExists
    ? await crypto.subtle.importKey(
      "raw",
      await file.arrayBuffer(),
      ENCRYPTION_ALGORITHM,
      true,
      usages,
    )
    : await crypto.subtle.generateKey(ENCRYPTION_ALGORITHM, true, usages);

  if (!fileExists) {
    Bun.write(filePath, await crypto.subtle.exportKey("raw", secret), {
      mode: 400,
    });
  }

  return secret;
}

export const API_KEY_ENCRYPTION_SECRET = await getEncryptionSecret(
  ".api-key-encryption-secret",
);

export const AUTH_SECRET = new Uint8Array(
  await crypto.subtle.exportKey(
    "raw",
    await getEncryptionSecret(".auth-secret"),
  ),
).toHex();

export async function decryptHexEncodedApiKey(
  hexEncodedInitializationVector: string,
  encryptionSecret: CryptoKey,
  hexEncodedEncryptedApiKey: string,
): Promise<ArrayBuffer> {
  const decrypt = await crypto.subtle.decrypt(
    {
      iv: Uint8Array.fromHex(hexEncodedInitializationVector),
      ...ENCRYPTION_ALGORITHM,
    },
    encryptionSecret,
    Uint8Array.fromHex(hexEncodedEncryptedApiKey),
  );

  return decrypt;
}
