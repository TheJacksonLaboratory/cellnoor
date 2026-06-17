import { auth } from "$lib/auth";
import { svelteKitHandler } from "better-auth/svelte-kit";
import { building } from "$app/env";
import { redirect } from "@sveltejs/kit";
import { PUBLIC_AUTH_URL } from "$app/env/public";

const NON_AUTH_ROUTES = ["/health"];

function requiresAuth(path: string) {
  return !NON_AUTH_ROUTES.some((s) => path.includes(s));
}

export async function handle({ event, resolve }) {
  const {
    url: { pathname },
    request: { headers },
  } = event;
  if (!requiresAuth(pathname)) {
    return svelteKitHandler({ event, resolve, auth, building });
  }

  const session = await auth.api.getSession({
    headers,
  });

  if (!session) {
    const authUrl = new URL(PUBLIC_AUTH_URL ?? "");
    authUrl.searchParams.append("redirect_to", event.url.toString());

    return redirect(307, authUrl);
  }

  // We could destructure this (the way we do with other things above), but I'm not sure if `session` is a reference or
  // a deep clone because JavaScript <3
  event.locals.user = session.user;

  return svelteKitHandler({ event, resolve, auth, building });
}
