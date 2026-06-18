import createClient from "openapi-fetch";
import type { Client, ClientOptions, Middleware } from "openapi-fetch";
import type { paths } from "$lib/cellnoor-types";
import { getRequestEvent } from "$app/server";
import { API_SOCKET, API_URL } from "$app/env/private";

export type CellnoorClient = Client<paths>;

const apiClient: CellnoorClient | null = null;

function createCellnoorClient(options?: ClientOptions) {
  return createClient<paths>(options);
}

export function getApiClient() {
  if (apiClient !== null) {
    return apiClient;
  }

  const client = createCellnoorClient({
    baseUrl: API_URL,
    fetch: async (request) => {
      return fetch(request, { unix: API_SOCKET });
    },
  });

  client.use(middleware);

  return client;
}

const middleware: Middleware = {
  async onRequest({ request }) {
    const { cookies } = getRequestEvent();

    // We have to manually copy cookies since we're not using SvelteKit's fetch (because only Bun's works with Unix domain sockets)
    const securePrefix = "__Secure-";
    const authCookieName = "cellnoor-auth.session_data";
    const secureCookieName = `${securePrefix}${authCookieName}`;

    for (const {name, value} of cookies.getAll()) {
      if (name.startsWith(authCookieName) || name.startsWith(secureCookieName)) {
        request.headers.append("Cookie", `${name}=${value}`);
      }
    }
  },
};
