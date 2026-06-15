import type { ChromiumDatasetDetailed, ProjectCompact } from "$lib/cellnoor-types";
import { getApiClient } from "$lib/server/cellnoor-client";

export async function load({
  params: { id },
}): Promise<{ dataset: ChromiumDatasetDetailed; project: ProjectCompact } | { error: string }> {
  const apiClient = getApiClient();

  const params = { path: { id } };

  const error = { error: "something went wrong" };

  const { data: dataset } = await apiClient.GET("/chromium-datasets/{id}", { params });
  if (!dataset) {
    return error;
  }

  // Use projects/search instead of projects/{id} because the former is a lighter operation
  const { data: projects } = await apiClient.POST("/projects/search", {
    body: { filter: { id: dataset.specimens[0]?.project_id! } },
  });

  if (!projects) {
    return error;
  }

  return { dataset, project: projects[0]! };
}
