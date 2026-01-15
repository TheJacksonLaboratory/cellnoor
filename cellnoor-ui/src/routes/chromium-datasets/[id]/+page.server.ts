import { ApiClient } from "$lib/server/cellnoor-client";
import type { ChromiumDataset } from "cellnoor-types/ChromiumDataset";
import type { SpecimenSummary } from "cellnoor-types/SpecimenSummary";
import type { LibrarySummary } from "cellnoor-types/LibrarySummary";
import { isSuccess } from "$lib/cellnoor-typeguard";

export async function load(
  event,
) {
  const apiClient = await ApiClient.new();

  const dataset = await apiClient.getJson<ChromiumDataset>();

  const [specimens, libraries] = isSuccess(dataset)
    ? await Promise.all([
      apiClient.getJson<SpecimenSummary>({
        endpoint: dataset.links.specimens as string,
        queryString: "",
      }),
      apiClient.getJson<LibrarySummary>({
        endpoint: dataset.links.libraries as string,
        queryString: "",
      }),
    ])
    : [undefined, undefined];

  return {
    dataset,
    specimens,
    libraries,
  };
}
