import { building } from "$app/environment";

// This module could be improved but I hate writing TypeScript so it's not worth it
async function readEnvVar(name: string) {
  const key = name.toUpperCase();
  const val = Bun.env[key] || Bun.env[`CELLNOOR_${key}`];

  return val;
}

async function readRequiredEnvVar(name: string) {
  const val = await readEnvVar(name);

  if (val === undefined) {
    throw `required environment variable ${name} is unset`;
  }

  return val;
}

async function readSecret(name: string): Promise<string> {
  if (Bun.env.IN_DOCKER?.toLowerCase() === "true") {
    return await Bun.file(`/run/secrets/${name}`).text();
  }

  return await readRequiredEnvVar(name);
}

interface Secrets {
  dbHost: string;
  dbPort: number;
  cellnoorUiDbPassword: string;
  dbName: string;
  authSecret: string;
  microsoftEntraClientId: string;
  microsoftEntraClientSecret: string;
  microsoftEntraTenant: string;
}

let secrets: Secrets | null = null;

export async function readSecrets() {
  if (building) {
    return {
      dbHost: "",
      dbPort: 0,
      cellnoorUiDbPassword: "",
      dbName: "",
      authSecret: "",
      microsoftEntraClientId: "",
      microsoftEntraClientSecret: "",
      microsoftEntraTenant: "",
    };
  }

  if (secrets !== null) {
    return secrets;
  }

  secrets = {
    dbHost: await readRequiredEnvVar("db_host"),
    dbPort: parseInt(await readRequiredEnvVar("db_port")),
    cellnoorUiDbPassword: await readSecret("cellnoor_ui_db_password"),
    dbName: await readSecret("db_name"),
    authSecret: await readSecret("auth_secret"),
    microsoftEntraClientId: await readSecret("microsoft_entra_client_id"),
    microsoftEntraClientSecret: await readSecret("microsoft_entra_client_secret"),
    microsoftEntraTenant: await readSecret("microsoft_entra_tenant"),
  };

  return secrets;
}

interface Config {
  publicUrl: string;
  apiUrl: string;
  apiSocket?: string;
  jwtAudience: string;
  jwtIssuer: string;
}

let appConfig: Config | null = null;

export async function readConfig() {
  if (building) {
    return { publicUrl: "", apiUrl: "", jwtAudience: "", jwtIssuer: "" };
  }

  if (appConfig !== null) {
    return appConfig;
  }

  const [publicUrl, apiUrl] = await Promise.all(["public_url", "api_url"].map(readRequiredEnvVar));
  const apiSocket = await readEnvVar("api_socket");

  const [jwtAudience, jwtIssuer] = await Promise.all(
    ["jwt_audience", "jwt_issuer"].map(readRequiredEnvVar),
  );

  appConfig = {
    publicUrl,
    apiUrl,
    apiSocket,
    jwtAudience,
    jwtIssuer,
  } as Config;

  return appConfig;
}
