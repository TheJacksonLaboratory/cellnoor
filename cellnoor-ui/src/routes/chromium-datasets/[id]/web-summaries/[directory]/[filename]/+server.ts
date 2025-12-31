import { ApiClient } from "$lib/server/cellnoor-client";
import type { RequestHandler } from "./$types";

// It would be faster to just grab the web summary from the database and send
// it directly. However, that requires us to do `set local role = ${userID}`
// to take advantage of Postgres's row-level security. Unfortunately, that
// requires the `cellnoor-ui` database user to be granted all user roles.
// That might be a vulenerability because I don't trust myself in JavaScript.
export const GET: RequestHandler = async (event) => {
  const apiClient = await ApiClient.new();

  return await apiClient.get(event);
};
