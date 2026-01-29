import { getApiClient } from "$lib/server/cellnoor-client";

export async function load() {
  const apiClient = await getApiClient();

  const [chromiumDatasets, assays] = await Promise.all([
    apiClient.GET("/chromium-datasets"),
    apiClient.GET("/10x-assays"),
  ]);

  return {
    chromiumDatasets,
    assays,
  };
}
