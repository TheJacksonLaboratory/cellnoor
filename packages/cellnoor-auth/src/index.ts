import { auth } from "./auth";
import { readConfig } from "./config";
import signIn from "./routes/sign-in/index.html";

const { unixDomainSocket, publicAppUrl } = await readConfig();

// We don't need to pass the host or port because Bun automatically picks them up from the environment if set
Bun.serve({
  unix: unixDomainSocket,
  routes: {
    "/": signIn,
    "/sign-in": signIn,
    "/api/auth/*": auth.handler,
  },
});
