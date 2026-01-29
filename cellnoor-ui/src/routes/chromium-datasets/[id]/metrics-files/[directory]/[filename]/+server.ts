import { ApiClient } from "$lib/server/cellnoor-client-dep";

export async function GET() {
  const apiClient = await ApiClient.new();
  return apiClient.get();
}
