import { ApiClient } from "$lib/server/cellnoor-client";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async (event) => {
  const apiClient = await ApiClient.new();

  return await apiClient.get(event, {
    headers: { accept: "text/csv", "X-API-Key": event.locals.apiKey },
  });
};
