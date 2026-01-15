import { ApiClient } from "$lib/server/cellnoor-client";
import type { RequestHandler } from "./$types";

const apiClient = await ApiClient.new();
export const GET = apiClient.get;
