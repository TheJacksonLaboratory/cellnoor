import { betterAuth } from "better-auth";

// We only use better-auth in this app to make it easier to get user information (rather than parsing the JWT ourselves)
export const auth = betterAuth({
  user: {
    additionalFields: {
      is_staff: {type: "boolean"}
    },
  },
  advanced: {
    cookiePrefix: "cellnoor-auth",
  },
});
