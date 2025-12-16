import { ApiClient } from "$lib/server/scamplers-client";
import type { ChromiumDataset } from "scamplers-types/ChromiumDataset";
import type { PageServerLoad } from "./$types";
import type { SpecimenSummary } from "scamplers-types/SpecimenSummary";
import type { LibrarySummary } from "scamplers-types/LibrarySummary";

export const load: PageServerLoad = async (
  event,
) => {
  const apiClient = await ApiClient.new();

  const dataset = await apiClient.get<ChromiumDataset>(event);
  const [specimens, libraries] = await Promise.all([
    apiClient.get<SpecimenSummary>(event, dataset.links.specimens as string),
    apiClient.get<LibrarySummary>(event, dataset.links.libraries as string),
  ]);

  return {
    dataset,
    specimens,
    libraries,
  };
};
