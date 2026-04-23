import { auth } from "./auth";
import { readConfig } from "./config";
import signInPage from "./sign-in.html";

const { unixDomainSocket } = await readConfig();

// We don't need to pass the host or port because Bun automatically picks them up from the environment if set
Bun.serve({
  unix: unixDomainSocket,
  routes: {
    "/sign-in": signInPage,
    "/api/auth/*": auth.handler,
  },
});
