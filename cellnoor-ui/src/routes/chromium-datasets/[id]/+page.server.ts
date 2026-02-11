import { getApiClient } from "$lib/server/cellnoor-client.js";

export async function load({ params: { id } }) {
  const apiClient = await getApiClient();

  const params = { path: { id } };

  const [dataset, specimens, libraries] = await Promise.all([
    apiClient.GET("/chromium-datasets/{dataset_id}", {
      params,
    }),
    apiClient.GET("/chromium-datasets/{dataset_id}/specimens", { params }),
    apiClient.GET("/chromium-datasets/{dataset_id}/libraries", { params }),
  ]);

  return {
    dataset: dataset.data,
    specimens: specimens.data,
    libraries: libraries.data,
  };
}
