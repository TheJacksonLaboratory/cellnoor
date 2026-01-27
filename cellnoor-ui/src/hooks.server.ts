import { auth } from "$lib/auth";
import { svelteKitHandler } from "better-auth/svelte-kit";
import { building } from "$app/environment";
import { redirect } from "@sveltejs/kit";
import { API_TOKEN_COOKIE_NAME } from "$lib/server/cellnoor-client";
import createClient from "openapi-fetch";
import { paths } from "$lib/server/cellnoor-client3";

const NON_AUTH_ROUTES = [
  "/api/auth/sign-in/social",
  "/api/auth/callback/microsoft",
  "/api/auth/jwks",
  "/auth/sign-in",
  "/health",
];

function requiresAuth(path: string) {
  return !NON_AUTH_ROUTES.some((s) => path.includes(s));
}

export async function handle({ event, resolve }) {
  const client = createClient<paths>({ baseUrl: "http://localhost:8000/api", querySerializer: (q) => `query=${JSON.stringify(q.query)}` });
  client.GET("/institutions", {params: {query: {query: {names: [""]}}}})

  const { url: { pathname }, request: { headers } } = event;
  if (!requiresAuth(pathname)) {
    return svelteKitHandler({ event, resolve, auth, building });
  }

  const session = await auth.api.getSession({
    headers,
  });

  if (!session) {
    return redirect(307, "/auth/sign-in");
  }

  const userIsSingingOut = pathname.includes("sign-out");
  if (userIsSingingOut) {
    event.cookies.delete(API_TOKEN_COOKIE_NAME, { path: "/" });
  }

  // We could destructure this (the way we do with other things above), but I'm not sure if `session` is a reference or
  // a deep clone because JavaScript <3
  event.locals.user = session.user;

  return svelteKitHandler({ event, resolve, auth, building });
}
