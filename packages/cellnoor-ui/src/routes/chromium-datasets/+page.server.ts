import type {
  ChromiumDatasetCompact,
  ChromiumDatasetDetailed,
  ProjectCompact,
  TenxAssay,
} from "$lib/cellnoor-types";
import { getApiClient } from "$lib/server/cellnoor-client";

type ReturnType =
  | {
      datasets: ChromiumDatasetDetailed[];
      assays: TenxAssay[];
      projects: ProjectCompact[];
    }
  | { error: string };

export async function load({ url }) {
  return await loadData(url.searchParams.get("q") || `{"limit": 5000}`);
}

async function loadData(q?: string): Promise<ReturnType> {
  const client = getApiClient();

  // We don't care whatsoever what the actual query is because the API will validate it. Since the client is type-safe anyways this is fine
  const getDatasets = client.POST("/chromium-datasets/search/detailed", {
    body: q ? JSON.parse(q) : {},
  });

  const [chromiumDatasets, assays, projects] = await Promise.all([
    getDatasets,
    client.GET("/10x-assays"),
    client.GET("/projects"),
  ]);

  if (!chromiumDatasets.data || !assays.data || !projects.data) {
    return { error: "something went wrong" };
  }

  return {
    datasets: chromiumDatasets.data,
    assays: assays.data,
    projects: projects.data,
  };
}
