import { auth } from "./auth";
import { svelteKitHandler } from "better-auth/svelte-kit";
import { building } from "$app/environment";
import { redirect } from "@sveltejs/kit";

const NON_AUTH_ROUTES = ["/auth/sign-in", "/health", "/api/auth"];

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

  event.locals.user = session.user;

  return svelteKitHandler({ event, resolve, auth, building });
}
