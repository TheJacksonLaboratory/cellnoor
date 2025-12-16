import { ApiClient } from "$lib/server/scamplers-client";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async (event) => {
  const apiClient = await ApiClient.new();

  if (event.request.url.endsWith(".csv")) {
    event.request.headers.set("Accept", "text/csv");
  }

  return await apiClient.getRaw(event);
};
