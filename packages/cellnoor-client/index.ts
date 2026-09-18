import createClient from "openapi-fetch";
import type { Client, ClientOptions } from "openapi-fetch";
import type { paths } from "./cellnoor-types";

export type CellnoorClient = Client<paths>;

export function createCellnoorClient(options?: ClientOptions) {
  return createClient<paths>(options);
}
