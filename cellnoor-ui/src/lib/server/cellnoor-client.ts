import type { RequestEvent, ServerLoadEvent } from "@sveltejs/kit";
import { readConfig } from "$lib/server/config";
import type { ApiErrorResponse } from "cellnoor-types/ApiErrorResponse";
import { auth } from "../../auth";
import * as jose from "jose";

let apiClient: ApiClient | null = null;

const API_TOKEN_COOKIE_NAME = "cellnoor-ui.api_token";
export const jwks = await auth.api.getJwks().then((keys) =>
  jose.createLocalJWKSet(keys)
);

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

  private getApiTokenFromCookies(event: ServerLoadEvent | RequestEvent) {
    return event.cookies.get(API_TOKEN_COOKIE_NAME);
  }

  private async setNewApiToken(event: ServerLoadEvent | RequestEvent) {
    const { headers } = event.request;

    const { token: newToken } = await auth.api.getToken({ headers });
    auth.api.getToken({ headers });
    const { exp } = jose.decodeJwt(newToken);

    event.cookies.set(API_TOKEN_COOKIE_NAME, newToken, {
      path: "/",
      expires: new Date(exp! * 1000),
    });
  }

  private async refreshApiToken(
    event: ServerLoadEvent | RequestEvent,
    apiToken: string,
  ) {
    try {
      await jose.jwtVerify(apiToken, jwks);
    } catch (error) {
      if (!(error instanceof jose.errors.JWTExpired)) {
        throw error;
      }

      await this.setNewApiToken(event);
    }
  }

  private async authenticate(event: ServerLoadEvent | RequestEvent) {
    let apiToken = this.getApiTokenFromCookies(event);

    if (apiToken === undefined) {
      await this.setNewApiToken(event);
    } else {
      await this.refreshApiToken(event, apiToken);
    }
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
    await this.authenticate(event);

    return await event.fetch(apiUrl, requestData);
  }

  async get(
    event: ServerLoadEvent | RequestEvent,
    requestData: RequestInit = { method: "GET" },
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
      headers: { accept: "application/json" },
    },
    { endpoint, queryString }: {
      endpoint: string;
      queryString: string;
    } = { endpoint: event.url.pathname, queryString: event.url.search },
  ): Promise<T | ApiErrorResponse> {
    const response = await this.get(
      event,
      requestData,
      {
        endpoint,
        queryString,
      },
    );
    const asJson = await response.json();

    return asJson;
  }
}
