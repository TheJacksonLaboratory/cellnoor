import { query } from "$app/server";
import { getApiClient } from "$lib/server/cellnoor-client";
import * as v from "valibot";

export const getChromiumDatasets = query(v.string(), fetchChromiumDatasets);

async function fetchChromiumDatasets(q: string) {
  const client = getApiClient();

  // We don't care whatsoever what the actual query is because the API will validate it. Since the client is type-safe anyways this is fine
  const { data: datasets } = await client.POST("/chromium-datasets/search", {
    body: JSON.parse(q),
  });

  if (!datasets) {
    return { error: "something went wrong" };
  }

  return datasets;
}
