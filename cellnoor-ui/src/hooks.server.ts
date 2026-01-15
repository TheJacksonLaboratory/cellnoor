import { auth } from "./auth";
import { svelteKitHandler } from "better-auth/svelte-kit";
import { building } from "$app/environment";
import { redirect } from "@sveltejs/kit";

const NON_AUTH_ROUTES = ["/auth/sign-in", "/health", "/api/auth"];

function needsAuth(path: string) {
  return !NON_AUTH_ROUTES.some((s) => path.includes(s));
}

export async function handle({ event, resolve }) {
  if (!needsAuth(event.url.pathname)) {
    return svelteKitHandler({ event, resolve, auth, building });
  }

  const headers = event.request.headers;

  const session = await auth.api.getSession({
    headers,
  });

  if (!session) {
    return redirect(307, "/auth/sign-in");
  }

  event.locals.session = session;

  return svelteKitHandler({ event, resolve, auth, building });
}
