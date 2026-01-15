import { auth } from "../../auth.js";
import * as jose from "jose";
import { getDbClient } from "$lib/server/db-client.js";
import { getUserApiTokens, insertApiToken } from "$lib/server/auth/db.js";

async function createNewApiToken(userId: string, expiresOn: Date) {
  const payload: jose.JWTPayload = {
    sub: userId,
    jti: Bun.randomUUIDv7(),
    exp: Math.floor(expiresOn.getTime() / 1000),
    iat: Math.floor(Date.now() / 1000),
  };

  const { token } = await auth.api.signJWT({ body: { payload } });

  return token;
}

function oneYearFromNow() {
  const now = new Date();
  now.setFullYear(now.getFullYear() + 1);

  return now;
}

function justGetTheDamnDate(date: Date) {
  return date.toISOString().split("T")[0]!;
}

function validateFormData(data: FormData) {
  const name = data.get("name");
  const description = data.get("description");
  const expiresOnStr = data.get("expiresOn");

  if ((!expiresOnStr) || (!name)) {
    return { error: "API token name and expiration date must be set" };
  }

  const expiresOn = new Date(expiresOnStr.toString());
  if (expiresOn > oneYearFromNow()) {
    return {
      error: "Expiration date cannot be more than one year into the future",
    };
  }

  return {
    name: name.toString(),
    description: description ? description.toString() : null,
    expiresOn,
  };
}

export const actions = {
  createApiToken: async ({ request, locals: { user: { userId } } }) => {
    const { name, description, expiresOn, error } = validateFormData(
      await request.formData(),
    );

    if (error) {
      return { error };
    }

    const apiToken = await createNewApiToken(userId, expiresOn!);
    const { jti, sub, iat } = jose.decodeJwt(apiToken);
    const apiTokenForDb = {
      jti: jti!,
      sub: sub!,
      name: name!.toString(),
      description: description ? description.toString() : null,
      exp: expiresOn!,
      iat: new Date(iat! * 1000),
    };

    const dbClient = await getDbClient();
    await insertApiToken(apiTokenForDb, dbClient);

    return { apiToken };
  },
  deleteApiToken: async ({ request: { formData } }) => {
    // TODO:
    // 1. delete the API token from the table api_tokens so that it's no longer displayed for the user
    // 2. add the API token to the `revoked_tokens` table so that the backend knows not to accept it
  },
};

export async function load({ locals: { user: { userId } } }) {
  const dbClient = await getDbClient();
  const apiTokens = await getUserApiTokens(userId, dbClient);

  return {
    apiTokens,
    today: justGetTheDamnDate(new Date()),
    oneYearFromNow: justGetTheDamnDate(oneYearFromNow()),
  };
}
