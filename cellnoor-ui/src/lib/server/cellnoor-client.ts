import { readConfig } from "$lib/server/config";
import { auth } from "../auth";
import * as jose from "jose";
import { getRequestEvent } from "$app/server";
import { type CellnoorClient, createCellnoorClient } from "cellnoor-client";
import type { Middleware } from "openapi-fetch";

let apiClient: CellnoorClient | null = null;

export async function getApiClient() {
  if (apiClient !== null) {
    return apiClient;
  }

  const baseUrl = await readConfig().then(({ apiUrl }) => apiUrl);
  const client = createCellnoorClient({
    baseUrl,
    fetch: getRequestEvent().fetch,
    compressRequests: true, // Enable request compression
  });

  client.use(middleware);

  return client;
}

const middleware: Middleware = {
  async onRequest({ request }) {
    await reauthenticate();

    const { apiUrl, publicUrl } = await readConfig();
    const { cookies } = getRequestEvent();

    if (!apiUrl.startsWith(publicUrl || "")) {
      request.headers.set(
        "Cookie",
        `${API_TOKEN_COOKIE_NAME}=${cookies.get(API_TOKEN_COOKIE_NAME)}`,
      );
    }
  },
};

export async function reauthenticate() {
  let apiToken = await getApiTokenFromCookies();

  if (!apiToken) {
    await setNewApiToken();
  } else {
    await refreshApiToken(apiToken);
  }
}

async function getApiTokenFromCookies() {
  const { cookies } = getRequestEvent();
  return cookies.get(API_TOKEN_COOKIE_NAME);
}

async function setNewApiToken() {
  const { request: { headers }, cookies } = getRequestEvent();

  const { token: newToken } = await auth.api.getToken({ headers });
  const { exp } = jose.decodeJwt(newToken);

  cookies.set(API_TOKEN_COOKIE_NAME, newToken, {
    path: "/",
    expires: new Date(exp! * 1000),
    secure: true,
    sameSite: "strict",
    httpOnly: true,
  });
}

async function refreshApiToken(
  apiToken: string,
) {
  // We don't actually need to verify the JWT because the REST API will do that for us
  const { exp } = jose.decodeJwt(apiToken);
  if ((exp! * 1000) < Date.now()) {
    await setNewApiToken();
  }
}

export const API_TOKEN_COOKIE_NAME = "cellnoor-ui.api_token";
