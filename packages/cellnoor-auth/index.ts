import { auth } from "./auth";
import { readConfig } from "./config";

const { unixDomainSocket } = await readConfig();

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
    "/api/auth/*": async (request) => {
      const response = await auth.handler(request);

      return response;
    },
  },
});
