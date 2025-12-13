import type { ServerLoadEvent } from "@sveltejs/kit";
import { hexEncodedApiKeyFromCookies } from "./auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "./auth/crypto";
import { readConfig } from "$lib/server/config";
import qs from "qs";

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
    { cookies, fetch, url }: ServerLoadEvent,
    {
      endpoint,
      method,
      data,
    }: { endpoint?: string; method: string; data?: unknown },
  ): Promise<T> {
    if (!endpoint) {
      endpoint = url.pathname;
    }
    console.log(qs.parse(url.search.replace("?", "")));

    let apiUrl = `${this.apiBaseUrl}${endpoint}${url.search}`;
    if (!url.search.includes("limit=")) {
      apiUrl = `${apiUrl}&limit=50`;
    }

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

    const response = await fetch(apiUrl, options);
    const asJson = await response.json();

    if (asJson.error) {
      throw asJson;
    }

    return asJson;
  }

  async get<T>(event: ServerLoadEvent, endpoint?: string): Promise<T> {
    return await this.sendRequest(event, { endpoint, method: "GET" });
  }

  async post<T>(
    event: ServerLoadEvent,
    endpoint: string,
    data: unknown,
  ): Promise<T> {
    return await this.sendRequest(event, { endpoint, method: "POST", data });
  }
}
