import type { Cookies, RequestEvent, ServerLoadEvent } from "@sveltejs/kit";
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

  private constructUrl(
    { endpoint, queryString }: { endpoint: string; queryString: string },
  ): string {
    if (!queryString) {
      queryString = "?";
    }

    if (!queryString.includes("limit=")) {
      queryString = `${queryString}&limit=50`;
    }

    return `${this.apiBaseUrl}${endpoint}${queryString}`;
  }

  private async sendRequest(
    event: ServerLoadEvent | RequestEvent,
    requestData: RequestInit,
    { endpoint, queryString }: {
      endpoint: string;
      queryString: string;
    },
  ): Promise<Response> {
    const apiUrl = this.constructUrl({ endpoint, queryString });

    return await event.fetch(apiUrl, requestData);
  }

  async get(
    event: ServerLoadEvent | RequestEvent,
    requestData: RequestInit = {
      method: "GET",
      headers: { "X-API-Key": event.locals.apiKey },
    },
    { endpoint, queryString }: {
      endpoint: string;
      queryString: string;
    } = { endpoint: event.url.pathname, queryString: event.url.search },
  ): Promise<Response> {
    return await this.sendRequest(
      event,
      requestData,
      {
        endpoint,
        queryString,
      },
    );
  }

  async getJson<T>(
    event: ServerLoadEvent | RequestEvent,
    requestData: RequestInit = {
      method: "GET",
      headers: { accept: "application/json", "X-API-Key": event.locals.apiKey },
    },
    { endpoint, queryString }: {
      endpoint: string;
      queryString: string;
    } = { endpoint: event.url.pathname, queryString: event.url.search },
  ): Promise<T> {
    const response = await this.get(
      event,
      requestData,
      {
        endpoint,
        queryString,
      },
    );
    const asJson = await response.json();

    if (asJson.error) {
      throw new Error(JSON.stringify(asJson));
    }

    return asJson;
  }
}
