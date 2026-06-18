import { auth, trustedOrigins } from "./auth";
import { readConfig } from "./config";
import signIn from "./routes/sign-in/sign-in.html";

const { unixDomainSocket } = await readConfig();

const allowedOrigins = new Set(trustedOrigins);

Bun.serve({
  unix: unixDomainSocket,
  routes: {
    "/sign-in": signIn,
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
      }
    },
    // We need to add some cross-origin bullshit in order to call /sign-out from app.cellnoor.jax.org
    "/api/auth/*": async request => {
      const origin = request.headers.get("origin");
      const allowed = origin !== null && allowedOrigins.has(origin);

      // better-auth doesn't handle OPTIONS requests so we have to catch it ourselves
      if (request.method === "OPTIONS") {
        const headers = new Headers();
        if (allowed) {
          headers.set("Access-Control-Allow-Origin", origin);
          headers.set("Access-Control-Allow-Credentials", "true");
          headers.set(
            "Access-Control-Allow-Headers",
            request.headers.get("access-control-request-headers") ?? "",
          );
        }

        return new Response(null, { status: 204, headers });
      }

      const response = await auth.handler(request);

      if (allowed) {
        response.headers.set("Access-Control-Allow-Origin", origin);
        response.headers.set("Access-Control-Allow-Credentials", "true");
      }
      return response;
    },
  },
  async fetch({ url }) {
    const requestUrl = new URL(url);

    return Response.redirect(`/sign-in${requestUrl.search}`);
  },
});
