import { auth } from "./auth";
import { svelteKitHandler } from "better-auth/svelte-kit";
import { building } from "$app/environment";
import { redirect } from "@sveltejs/kit";

const NON_AUTH_ROUTES = [
  "/api/auth/sign-in/social",
  "/api/auth/callback/microsoft",
  "/auth/sign-in",
  "/health",
];

function requiresAuth(path: string) {
  return !NON_AUTH_ROUTES.some((s) => path.includes(s));
}

export async function handle({ event, resolve }) {
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

  // We could destructure this (the way we do with other things above), but I'm not sure if `session` is a reference or
  // a deep clone
  event.locals.user = session.user;

  return svelteKitHandler({ event, resolve, auth, building });
}
