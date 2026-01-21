import { ApiClient } from "$lib/server/cellnoor-client";
import type { ChromiumDataset } from "cellnoor-types/ChromiumDataset";
import type { SpecimenSummary } from "cellnoor-types/SpecimenSummary";
import type { LibrarySummary } from "cellnoor-types/LibrarySummary";
import { isSuccess } from "$lib/cellnoor-typeguard";
import qs from "qs";
import type { ChromiumDatasetQuery } from "cellnoor-types/ChromiumDatasetQuery.js";

export async function load() {
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

export const actions = {
  search: async ({request}) => {
    const specimenName = await request.formData().then((d) => d.get("specimenName"));
    const query: ChromiumDatasetQuery = {filter: {specimen: {names: [`%${specimenName}%`]}}};

    qs.stringify(query, { encodeValuesOnly: true });
  }
}
