import type { RequestEvent, ServerLoadEvent } from "@sveltejs/kit";
import { readConfig } from "$lib/server/config";
import type { ApiErrorResponse } from "cellnoor-types/ApiErrorResponse";
import { auth } from "../auth";
import * as jose from "jose";
import { getRequestEvent } from "$app/server";

let apiClient: ApiClient | null = null;

export const API_TOKEN_COOKIE_NAME = "cellnoor-ui.api_token";

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

  private getApiTokenFromCookies() {
    const { cookies } = getRequestEvent();
    return cookies.get(API_TOKEN_COOKIE_NAME);
  }

  private async setNewApiToken() {
    const { request: { headers }, cookies } = getRequestEvent();

    const { token: newToken } = await auth.api.getToken({ headers });
    auth.api.getToken({ headers });
    const { exp } = jose.decodeJwt(newToken);

    cookies.set(API_TOKEN_COOKIE_NAME, newToken, {
      path: "/",
      expires: new Date(exp! * 1000),
      secure: true,
      sameSite: "strict",
      httpOnly: true,
    });
  }

  private async refreshApiToken(
    apiToken: string,
  ) {
    try {
      await auth.api.verifyJWT({ body: { token: apiToken } });
    } catch (error) {
      if (!(error instanceof jose.errors.JWTExpired)) {
        throw error;
      }

      await this.setNewApiToken();
    }
  }

  private async reauthenticate() {
    let apiToken = this.getApiTokenFromCookies();

    if (!apiToken) {
      await this.setNewApiToken();
    } else {
      await this.refreshApiToken(apiToken);
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
    { endpoint, queryString }: { endpoint: string; queryString: string },
    requestData: RequestInit,
  ): Promise<Response> {
    await this.reauthenticate();
    const apiUrl = this.constructUrl({ endpoint, queryString });

    const event = getRequestEvent();

    return await event.fetch(apiUrl, requestData);
  }

  async get(
    url?: { endpoint: string; queryString: string },
    requestData: RequestInit = { method: "GET" },
  ): Promise<Response> {
    if (!url) {
      const { url: { pathname, search } } = getRequestEvent();
      url = { endpoint: pathname, queryString: search };
    }

    return await this.sendRequest(
      url,
      requestData,
    );
  }

  async getJson<T>(
    url?: { endpoint: string; queryString: string },
    requestData: RequestInit = {
      method: "GET",
      headers: { accept: "application/json" },
    },
  ): Promise<T | ApiErrorResponse> {
    const response = await this.get(
      url,
      requestData,
    );
    const asJson = await response.json();

    return asJson;
  }
}
