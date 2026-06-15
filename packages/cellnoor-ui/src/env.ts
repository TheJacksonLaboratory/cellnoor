import { defineEnvVars } from "@sveltejs/kit/hooks";
import * as v from "valibot";

export const variables = defineEnvVars({
  // I do not understand why I need these all to be optional. That kind of defeats the fucking purpose doesn't it
  PUBLIC_AUTH_URL: { public: true, schema: v.optional(v.string()) },
  // It doesn't particularly matter what we set API_URL to because the app actually makes calls to the API over unix domain socket
  API_URL: { schema: v.optional(v.string(), "http://localhost") },
  API_SOCKET: { schema: v.optional(v.string()) },
  AUTH_SECRET: { schema: v.optional(v.string()) },
});
