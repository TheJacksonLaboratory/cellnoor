// This module could be improved but I hate writing TypeScript so it's not worth it
async function readEnvVar(name: string): Promise<string | undefined> {
  const key = name.toUpperCase();
  const val = Bun.env[key] || Bun.env[`CELLNOOR_${key}`];

  return val;
}

async function readRequiredEnvVar(name: string): Promise<string> {
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
  microsoft_entra_client_id: string;
  microsoft_entra_client_secret: string;
  microsoft_entra_tenant: string;
}

let secrets: Secrets | null = null;

export async function readSecrets() {
  if (secrets !== null) {
    return secrets;
  }

  secrets = {
    dbHost: await readRequiredEnvVar("db_host"),
    dbPort: parseInt(await readRequiredEnvVar("db_port")),
    cellnoorUiDbPassword: await readSecret("cellnoor_ui_db_password"),
    dbName: await readSecret("db_name"),
    microsoft_entra_client_id: await readSecret("microsoft_entra_client_id"),
    microsoft_entra_client_secret: await readSecret(
      "microsoft_entra_client_secret",
    ),
    microsoft_entra_tenant: await readSecret("microsoft_entra_tenant"),
  };

  return secrets;
}

interface Config {
  apiKeyPrefixLength: number;
  publicUrl?: string;
  apiUrl: string;
}

let appConfig: Config | null = null;

export async function readConfig() {
  if (appConfig !== null) {
    return appConfig;
  }

  const publicUrl = await readEnvVar("public_url");
  let apiUrl = await readEnvVar("api_url");

  if (apiUrl === undefined) {
    if (publicUrl === undefined) {
      throw "must have at least one of API_URL or PUBLIC_URL set";
    }

    apiUrl = `${publicUrl}/api`;
  }

  appConfig = {
    apiKeyPrefixLength: parseInt(
      await readRequiredEnvVar("api_key_prefix_length"),
    ),
    publicUrl,
    apiUrl,
  };

  return appConfig;
}
