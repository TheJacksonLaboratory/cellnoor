import { betterAuth } from "better-auth";
import {  readSecrets } from "$lib/server/secrets";

// We only use better-auth in this app to make it easier to get user information (rather than parsing the JWT ourselves)
export const auth = betterAuth({
  secret: await readSecrets().then(({ authSecret }) => authSecret),
  user: {
    additionalFields: {
      is_staff: {type: "boolean"}
    },
  },
  advanced: {
    cookiePrefix: "cellnoor-auth",
  },
});
