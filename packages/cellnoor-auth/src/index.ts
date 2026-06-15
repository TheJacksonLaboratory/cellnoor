import { auth } from "./auth";
import { readConfig } from "./config";
import signIn from "./routes/sign-in/sign-in.html";

const { unixDomainSocket } = await readConfig();

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
    "/api/auth/*": auth.handler,
  },
  async fetch() {
    return Response.redirect("/sign-in");
  },
});
