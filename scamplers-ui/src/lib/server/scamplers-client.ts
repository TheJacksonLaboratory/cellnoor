import type { Cookies, RequestEvent, ServerLoadEvent } from "@sveltejs/kit";
import { hexEncodedApiKeyFromCookies } from "./auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "./auth/crypto";
import { readConfig } from "$lib/server/config";

let apiClient: ApiClient | null = null;

type RequestSubset = {
  cookies: Cookies;
  fetch: typeof globalThis.fetch;
  url: URL;
  request: Request;
};

export class ApiClient {
  readonly apiBaseUrl: string;

  static async new(): Promise<ApiClient> {
    if (apiClient !== null) {
      return apiClient;
    }

    apiClient = new ApiClient((await readConfig()).apiUrl);

    return apiClient;
  }

  private constructor(apiBaseUrl: string) {
    this.apiBaseUrl = apiBaseUrl;
  }

  private async sendRequest(
    { cookies, fetch, url, request }: RequestSubset,
    endpoint?: string,
  ): Promise<Response> {
    const isSubRequest = endpoint !== undefined;

    if (!isSubRequest) {
      endpoint = url.pathname;
    }

    let queryString;
    if (isSubRequest) {
      // If this is a sub-request (like fetching assay-names when a request is made to /chromium-datasets), then the caller sets the query string
      queryString = "";
    } else {
      queryString = url.search ? url.search : "?";
      if (!queryString.includes("limit=")) {
        queryString = `${queryString}&limit=50`;
      }
    }

    let apiUrl = `${this.apiBaseUrl}${endpoint}${queryString}`;

    const apiKey = await hexEncodedApiKeyFromCookies(
      cookies,
      API_KEY_ENCRYPTION_SECRET,
    );

    if (apiKey) {
      request.headers.set("X-API-Key", apiKey);
    }

    return await fetch(apiUrl, request);
  }

  async getJson<T>(
    requestSubset: RequestSubset,
    endpoint?: string,
  ): Promise<T> {
    const response = await this.sendRequest(requestSubset, endpoint);
    const asJson = await response.json();
    if (response.status != 200) {
      throw new Error(JSON.stringify(asJson));
    }

    return asJson;
  }

  async getRaw(
    requestSubset: RequestSubset,
    endpoint?: string,
  ): Promise<Response> {
    return await this.sendRequest(requestSubset, endpoint);
  }
}
