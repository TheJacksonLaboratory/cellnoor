import { ENCRYPTION_ALGORITHM } from "./crypto";

const API_KEY_LENGTH = 32;
const INITIALIZATION_VECTOR_LENGTH = 12; // 96 bits (https://developer.mozilla.org/en-US/docs/Web/API/AesGcmParams#iv)

export class EncryptedApiKey {
  private readonly encryptedKey: Uint8Array;
  private readonly initializationVector: Uint8Array;
  readonly prefix: Uint8Array;
  readonly hash: string;

  private constructor({
    encryptedKey,
    initializationVector,
    prefix,
    hash,
  }: {
    encryptedKey: Uint8Array;
    initializationVector: Uint8Array;
    prefix: Uint8Array;
    hash: string;
  }) {
    this.encryptedKey = encryptedKey;
    this.initializationVector = initializationVector;
    this.prefix = prefix;
    this.hash = hash;
  }

  static async fromRandomValues(
    randomValues: Uint8Array<ArrayBuffer>,
    encryptionSecret: CryptoKey,
    apiKeyPrefixLength: number,
  ): Promise<EncryptedApiKey> {
    const initializationVector = crypto.getRandomValues(
      new Uint8Array(INITIALIZATION_VECTOR_LENGTH),
    );

    const encryptedApiKey = await crypto.subtle.encrypt(
      {
        iv: initializationVector,
        ...ENCRYPTION_ALGORITHM,
      },
      encryptionSecret,
      randomValues,
    );

    const prefix = randomValues.slice(0, apiKeyPrefixLength);
    const hash = await Bun.password.hash(randomValues);

    return new EncryptedApiKey({
      encryptedKey: new Uint8Array(encryptedApiKey),
      initializationVector,
      prefix,
      hash,
    });
  }

  static newUnencrypted(): Uint8Array<ArrayBuffer> {
    return crypto.getRandomValues(new Uint8Array(API_KEY_LENGTH));
  }

  static async new(
    encryptionSecret: CryptoKey,
    apiKeyPrefixLength: number,
  ): Promise<EncryptedApiKey> {
    const unencrypted = EncryptedApiKey.newUnencrypted();

    return await EncryptedApiKey.fromRandomValues(
      unencrypted,
      encryptionSecret,
      apiKeyPrefixLength,
    );
  }

  hexEncode(): string {
    return this.encryptedKey.toHex();
  }

  hexEncodedInitializationVector(): string {
    return this.initializationVector.toHex();
  }
}
