import { betterAuth } from "better-auth";
import { jwt } from "better-auth/plugins";
import { auth } from "../../auth.js";
import * as jose from "jose";
import { getDbClient } from "$lib/server/db-client.js";
import { getUserApiTokens } from "$lib/server/auth/db.js";

async function createNewApiToken(userId: string, expiresOn: Date) {
  const payload: jose.JWTPayload = {
    sub: userId,
    jti: Bun.randomUUIDv7(),
    exp: expiresOn.getTime() * 1000
  };

  const { token } = await auth.api.signJWT({ body: { payload } });

  return token;
}

export const actions = {
  createApiToken: async ({ request: { formData }, locals: { user: { userId } } }) => {
    const data = await formData();
    const expiresOn = data.get("expiresOn");
    console.log(expiresOn);

    return { apiToken: "foo" };

    const token = await createNewApiToken(userId, expiresOn!)
  },
  deleteApiToken: async ({ request: { formData } }) => {
    // Delete from api_tokens
    // Add token to revoked_tokens
    const data = await formData();
    console.log(data);
  }
};

export async function load({ locals: { user: {userId} } }) {
  const dbClient = await getDbClient();
  const apiTokens = await getUserApiTokens(userId, dbClient);

  return { apiTokens };
}
