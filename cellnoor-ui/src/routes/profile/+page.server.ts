import { betterAuth } from "better-auth";
import { jwt } from "better-auth/plugins";
import { auth } from "../../auth.js";
import * as jose from "jose";

async function createNewApiToken({ user }: typeof auth.$Infer.Session) {
  const payload: jose.JWTPayload = {
    sub: user.userId,
    jti: Bun.randomUUIDv7(),
    exp: Math.floor(Date.now() / 1000) + (365 * 24 * 60 * 60), // 1 year in seconds
  };

  const { token } = await auth.api.signJWT({ body: { payload } });

  return token;
}

// TODO: move the logic into actions
// https://svelte.dev/docs/kit/form-actions
export const actions = {};

export async function load(event) {
  const { headers } = event.request;
  const session = await auth.api.getSession({ headers });
  const token = await createNewApiToken(session!);

  return { token };
}
