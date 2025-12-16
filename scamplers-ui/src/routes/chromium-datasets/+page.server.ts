import { ApiClient } from "$lib/server/scamplers-client";
import type { ChromiumDatasetSummary } from "scamplers-types/ChromiumDatasetSummary";
import type { TenxAssay } from "scamplers-types/TenxAssay";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (
  event,
): Promise<
  { chromiumDatasets: ChromiumDatasetSummary[]; assays: TenxAssay[] }
> => {
  const apiClient = await ApiClient.new();

  const [chromiumDatasets, assays] = await Promise.all([
    apiClient.get<ChromiumDatasetSummary[]>(event),
    apiClient.get<TenxAssay[]>(event, "/10x-assays"),
  ]);

  // const chromiumDatasets = await Promise.all(datasetSummaries.map(async (ds) => { return { dataset: ds, specimens: await apiClient.get<SpecimenSummary[]>(event, ds.links.specimens as string) }; }))

  return {
    chromiumDatasets,
    assays,
  };
};
