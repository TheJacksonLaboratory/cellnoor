import { auth } from "./auth";
import { readConfig } from "./config";

const { unixDomainSocket } = await readConfig();

// Only paths on this site, so that these routes can't redirect anywhere else
function callbackPath(request: Request) {
  const base = "http://local.invalid";
  const callback = new URL(request.url).searchParams.get("callback") ?? "/";
  const url = new URL(callback, base);

  return url.origin === base ? `${url.pathname}${url.search}${url.hash}` : "/";
}

// Keeps the cookies better-auth set: the OAuth state when signing in, the cleared session when signing out
function redirect(response: Response, location: string, status: number) {
  const headers = new Headers({ location });

  for (const cookie of response.headers.getSetCookie()) {
    headers.append("set-cookie", cookie);
  }

  return new Response(null, { status, headers });
}

Bun.serve({
  unix: unixDomainSocket,
  routes: {
    "/api/refresh-accounts": {
      async POST(request) {
        const session = await auth.api.getSession(request);

        if (!session) {
          return new Response(null, { status: 401 });
        }

        // TODO:
        // 1. authorize that the user has necesssary permissions
        // 2. Fetch all MS user accounts from database
        // 3. Fetch user info from Microsoft Entra using `auth.api.getUserInfo` or equivalent
        return new Response();
      },
    },
    // The UI in cellnoor-api has no JavaScript for auth, so it signs in and out by navigating here
    // instead of calling better-auth's JSON endpoints
    "/api/auth/ui/sign-in": {
      async GET(request) {
        const response = await auth.api.signInSocial({
          body: { provider: "microsoft", callbackURL: callbackPath(request) },
          headers: request.headers,
          asResponse: true,
        });

        if (!response.ok) {
          return response;
        }

        const { url } = await response.json();

        return redirect(response, url, 302);
      },
    },
    "/api/auth/ui/sign-out": {
      async POST(request) {
        const response = await auth.api.signOut({
          headers: request.headers,
          asResponse: true,
        });

        return redirect(response, callbackPath(request), 303);
      },
    },
    "/api/auth/*": auth.handler,
  },
});
