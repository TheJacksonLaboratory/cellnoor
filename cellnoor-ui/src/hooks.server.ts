import { auth } from "./auth";
import { svelteKitHandler } from "better-auth/svelte-kit";
import { building } from "$app/environment";
import { type Cookies, redirect } from "@sveltejs/kit";
import { apiKeyFromCookies } from "$lib/server/auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "$lib/server/auth/crypto";

const NON_AUTH_ROUTES = ["/auth/sign-in", "/health", "/api/auth"];

async function hexEncodedApiKeyFromCookies(
  cookies: Cookies,
  encryptionSecret: CryptoKey,
): Promise<string | null> {
  const decryptedBytes = await apiKeyFromCookies(cookies, encryptionSecret);
  if (!decryptedBytes) {
    return null;
  }

  return new Uint8Array(decryptedBytes).toHex();
}

export async function handle({ event, resolve }) {
  if (NON_AUTH_ROUTES.some((s) => event.url.pathname.includes(s))) {
    return svelteKitHandler({ event, resolve, auth, building });
  }

  const session = await auth.api.getSession({
    headers: event.request.headers,
  });

  if (!session) {
    return redirect(307, "/auth/sign-in");
  }

  const hexEncodedApiKey = await hexEncodedApiKeyFromCookies(
    event.cookies,
    API_KEY_ENCRYPTION_SECRET,
  );
  event.locals.apiKey = hexEncodedApiKey ?? ""; // This kinda sucks :( but it's the simplest solution

  return svelteKitHandler({ event, resolve, auth, building });
}
