import createClient, { type Client, type ClientOptions } from "openapi-fetch";
import type { paths } from "./api";
export type * from "./api.d.ts";

export type CellnoorClient = Client<paths>;

export function createCellnoorClient(options?: ClientOptions) {
  return createClient<paths>(options);
}
