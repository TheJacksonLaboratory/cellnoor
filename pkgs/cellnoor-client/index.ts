import createClient, { type Client as InnerClient, type ClientOptions as InnerClientOptions } from "openapi-fetch";
import type { paths } from "./api";

export type ClientOptions = Omit<InnerClientOptions, "querySerializer">;

export type CellnoorClient = InnerClient<paths>;

  export function createCellnoorClient(options?: ClientOptions) {
  return createClient<paths>({
    ...options, querySerializer: (q) => `query=${JSON.stringify(q.query)}`})
}
