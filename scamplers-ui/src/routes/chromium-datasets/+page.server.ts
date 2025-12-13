import { ApiClient } from "$lib/server/scamplers-client";
import type { ChromiumDatasetSummary } from "scamplers-types/ChromiumDatasetSummary";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (
  event,
): Promise<{ chromiumDatasets: ChromiumDatasetSummary[] }> => {
  const apiClient = await ApiClient.new();

  return {
    chromiumDatasets: await apiClient.get(event),
  };
};
