import { ApiClient } from "$lib/server/cellnoor-client";
import type { ChromiumDataset } from "scamplers-types/ChromiumDataset";
import type { PageServerLoad } from "./$types";
import type { SpecimenSummary } from "scamplers-types/SpecimenSummary";
import type { LibrarySummary } from "scamplers-types/LibrarySummary";

export const load: PageServerLoad = async (
  event,
) => {
  const apiClient = await ApiClient.new();

  const dataset = await apiClient.getJson<ChromiumDataset>(event);
  const [specimens, libraries] = await Promise.all([
    apiClient.getJson<SpecimenSummary>(
      event,
      undefined,
      { endpoint: dataset.links.specimens as string, queryString: "" },
    ),
    apiClient.getJson<SpecimenSummary>(
      event,
      undefined,
      { endpoint: dataset.links.libraries as string, queryString: "" },
    ),
    ,
  ]);

  return {
    dataset,
    specimens,
    libraries,
  };
};
