import createClient from "openapi-fetch";
import type { Client, ClientOptions } from "openapi-fetch";
import type { DbError, paths } from "./cellnoor-types";

export type CellnoorClient = Client<paths>;
export type { paths };

export function createCellnoorClient(options?: ClientOptions) {
  return createClient<paths>(options);
}

function isDbError(error: {}): error is DbError {}
