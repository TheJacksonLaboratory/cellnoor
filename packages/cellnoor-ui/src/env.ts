import { defineEnvVars } from "@sveltejs/kit/hooks";
import * as v from "valibot";

export const variables = defineEnvVars({
  AUTH_SECRET: { schema: v.optional(v.string())},
  API_URL: {},
  API_SOCKET: { schema: v.optional(v.string()) },
});
