import { type ChromiumDatasetCompact, type ProjectCompact, type TenxAssay } from "$lib/cellnoor-types";
import { getApiClient } from "$lib/server/cellnoor-client";

type ReturnType =
  | {
      datasets: ChromiumDatasetCompact[],
      assays: TenxAssay[];
      projects: ProjectCompact[];
    }
  | { error: string };

export async function load({ url }) {
  return await loadData(url.searchParams.get("q") || `{"limit": 5000}`);
}

export const actions = {
  search: async ({ request }) => {
    const formData = await request.formData();
    return await loadData(formData.get("q")?.toString());
  },
};

async function loadData(q?: string): Promise<ReturnType> {
  const client = await getApiClient();

  // We don't care whatsoever what the actual query is because the API will validate it. Since the client is type-safe anyways this is fine
  const getDatasets = client.POST("/chromium-datasets/search", { body: q ? JSON.parse(q) : {} });

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
