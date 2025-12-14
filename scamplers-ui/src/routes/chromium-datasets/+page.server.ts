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
    apiClient.get(event),
    apiClient.get(event, "/10x-assays"),
  ]);

  return {
    chromiumDatasets,
    assays,
  };
};
