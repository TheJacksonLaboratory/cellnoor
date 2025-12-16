import type { RequestEvent, ServerLoadEvent } from "@sveltejs/kit";
import { hexEncodedApiKeyFromCookies } from "./auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "./auth/crypto";
import { readConfig } from "$lib/server/config";

let apiClient: ApiClient | null = null;

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

  private async sendRequest<T>(
    { cookies, fetch, url }: ServerLoadEvent | RequestEvent,
    {
      endpoint,
      method,
      data,
    }: { endpoint?: string; method: string; data?: unknown },
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

    const options: RequestInit = {
      method,
      headers: {
        "X-API-Key": apiKey || "",
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    };

    return await fetch(apiUrl, options);
  }

  async get<T>(event: ServerLoadEvent, endpoint?: string): Promise<T> {
    const response = await this.sendRequest(event, { endpoint, method: "GET" });
    const asJson = await response.json();

    if (asJson.error) {
      throw asJson;
    }

    return asJson;
  }

  async getRaw(event: RequestEvent, endpoint?: string): Promise<Response> {
    return await this.sendRequest(event, { endpoint, method: "GET" });
  }

  async post<T>(
    event: ServerLoadEvent,
    endpoint: string,
    data: unknown,
  ): Promise<T> {
    const response = await this.sendRequest(event, {
      endpoint,
      method: "POST",
      data,
    });
    const asJson = await response.json();

    if (asJson.error) {
      throw asJson;
    }

    return asJson;
  }
}
