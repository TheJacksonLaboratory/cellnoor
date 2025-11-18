import type { Institution } from "scamplers-types/Institution";
import type { ServerLoadEvent } from "@sveltejs/kit";
import { hexEncodedApiKeyFromCookies } from "./auth/cookies";
import { API_KEY_ENCRYPTION_SECRET } from "./auth/crypto";
import { readConfig } from "$lib/server/config";

class ApiClient {
  readonly apiBaseUrl: string;

  constructor(apiBaseUrl: string) {
    this.apiBaseUrl = apiBaseUrl;
  }

  private async sendRequest<T>(
    { cookies, fetch, url }: ServerLoadEvent,
    {
      endpoint,
      method,
      data,
    }: { endpoint: string; method: string; data?: unknown },
  ): Promise<T> {
    const apiUrl = `${this.apiBaseUrl}/${endpoint}?${url.search}`;

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

  private async get<T>(event: ServerLoadEvent, endpoint: string): Promise<T> {
    return await this.sendRequest(event, { endpoint, method: "GET" });
  }

  private async post<T>(
    event: ServerLoadEvent,
    endpoint: string,
    data: unknown,
  ): Promise<T> {
    return await this.sendRequest(event, { endpoint, method: "POST", data });
  }

  async listInstitutions(event: ServerLoadEvent): Promise<Institution[]> {
    return await this.get(event, "institutions");
  }
}

let apiClient: ApiClient | null = null;

export async function getApiClient(): Promise<ApiClient> {
  if (apiClient !== null) {
    return apiClient;
  }

  apiClient = new ApiClient((await readConfig()).apiUrl);

  return apiClient;
}
