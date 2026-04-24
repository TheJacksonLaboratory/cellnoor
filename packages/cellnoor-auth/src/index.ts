import { auth } from "./auth";
import { readConfig } from "./config";
import signIn from "./routes/sign-in/sign-in.html";

const { unixDomainSocket } = await readConfig();

Bun.serve({
  unix: unixDomainSocket,
  routes: {
    "/sign-in": signIn,
    "/api/auth/*": auth.handler,
  },
  async fetch({ url }) {
    return Response.redirect("/sign-in");
  },
});
