import { betterAuth } from "better-auth";
import { readSecrets } from "./server/secrets";

// We only use better-auth in this app to make it easier to get user information (rather than parsing the JWT ourselves)
export const auth = betterAuth({
  secret: await readSecrets().then((s) => s.authSecret),
  user: {
    additionalFields: {
      is_staff: { type: "boolean" },
      can_manage_users: { type: "boolean" },
    },
  },
  // This has to match cellnoor-auth
  session: {
    cookieCache: {
      enabled: true,
      strategy: "jwt",
      maxAge: 7 * 24 * 60 * 60,
      refreshCache: true,
    },
  },
  advanced: {
    cookiePrefix: "cellnoor-auth",
  },
});
