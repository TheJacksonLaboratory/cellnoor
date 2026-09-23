import createClient from "openapi-fetch";
import type { Client, ClientOptions } from "openapi-fetch";
import type {
  DbError,
  InstitutionPredicateQuery,
  paths,
} from "./cellnoor-types";

export type CellnoorClient = Client<paths>;
export type { paths };

export function createCellnoorClient(options?: ClientOptions) {
  return createClient<paths>(options);
}

async function f(x: CellnoorClient) {
  const y = await x.POST("/institutions/search", {
    body: { filter: { name: "" } },
  });

  const z: InstitutionPredicateQuery = { filter: { name: { like: "" } } };
}
