import { defineEnvVars } from "@sveltejs/kit/hooks";
import * as v from "valibot";

export const variables = defineEnvVars({
  API_URL: {},
  API_SOCKET: { schema: v.optional(v.string()) },
});
