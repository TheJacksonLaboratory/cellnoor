import { ApiClient } from "$lib/server/cellnoor-client";
import type { ChromiumDataset } from "scamplers-types/ChromiumDataset";
import type { PageServerLoad } from "./$types";
import type { SpecimenSummary } from "scamplers-types/SpecimenSummary";
import type { LibrarySummary } from "scamplers-types/LibrarySummary";
import type { ApiErrorResponse } from "scamplers-types/ApiErrorResponse";
import { isSuccess } from "$lib/cellnoor-typeguard";

export async function load(
  event,
) {
  const apiClient = await ApiClient.new();

  const dataset = await apiClient.getJson<ChromiumDataset>(event);

  const [specimens, libraries] = isSuccess(dataset)
    ? await Promise.all([
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
    ])
    : [undefined, undefined];

  return {
    dataset,
    specimens,
    libraries,
  };
}
