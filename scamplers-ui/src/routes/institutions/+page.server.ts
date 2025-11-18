import type { Institution } from "scamplers-types/Institution";
import { getApiClient } from "$lib/server/scamplers-client";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (event) => {
  const apiClient = await getApiClient();
  const institutions: Institution[] = await apiClient.listInstitutions(event);
  return { institutions };
};
