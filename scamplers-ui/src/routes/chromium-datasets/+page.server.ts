import { ApiClient } from "$lib/server/cellnoor-client";
import type { ChromiumDatasetSummary } from "scamplers-types/ChromiumDatasetSummary";
import type { ChromiumDatasetQuery } from "scamplers-types/ChromiumDatasetQuery";
import type { TenxAssay } from "scamplers-types/TenxAssay";
import type { PageServerLoad } from "./$types";
import qs from "qs";
import type { ApiErrorResponse } from "scamplers-types/ApiErrorResponse";

export async function load(event) {
  const apiClient = await ApiClient.new();

  const [chromiumDatasets, assays] = await Promise.all([
    apiClient.getJson<ChromiumDatasetSummary[]>(event),
    apiClient.getJson<TenxAssay[]>(event, undefined, {
      endpoint: "/10x-assays",
      queryString: "",
    }),
  ]);

  return {
    chromiumDatasets,
    assays,
  };
}
