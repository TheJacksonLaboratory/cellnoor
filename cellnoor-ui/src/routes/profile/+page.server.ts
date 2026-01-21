import { auth } from "$lib/auth";
import * as jose from "jose";
import { getDbClient } from "$lib/server/db-client.js";
import {
  deleteApiTokenFromDb,
  getUserJsonWebTokens,
  insertJsonWebToken,
} from "$lib/server/auth/db.js";

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

function sixMonthsFromNow() {
  const now = new Date();
  now.setMonth(now.getMonth() + 6);

  return now;
}

function justGetTheDamnDateAsAStringIHateThisLanguage(date: Date) {
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
  if (expiresOn > sixMonthsFromNow()) {
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
  createApiToken: async ({ request, locals: { user: { user_id } } }) => {
    const { name, description, expiresOn, error } = validateFormData(
      await request.formData(),
    );

    if (error) {
      return { error };
    }

    const apiToken = await createNewApiToken(user_id, expiresOn!);
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
    await insertJsonWebToken(apiTokenForDb, dbClient);

    return { apiToken };
  },
  deleteApiToken: async ({ request, locals: { user: { user_id } } }) => {
    const data = await request.formData();
    const jti = data.get("jti");

    if (!jti) {
      return;
    }

    await deleteApiTokenFromDb(
      { user_id, jti: jti.toString() },
      await getDbClient(),
    );
  },
};

export async function load({ locals: { user: { user_id } } }) {
  const dbClient = await getDbClient();
  const apiTokens = await getUserJsonWebTokens(user_id, dbClient);

  return {
    apiTokens,
    today: justGetTheDamnDateAsAStringIHateThisLanguage(new Date()),
    sixMonthsFromNow: justGetTheDamnDateAsAStringIHateThisLanguage(
      sixMonthsFromNow(),
    ),
  };
}
