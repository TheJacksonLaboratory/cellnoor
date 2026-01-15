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
  authSecret: string;
  jwtRs256PrivateKey: string;
  jwtRs256PublicKey: string;
  microsoftEntraClientId: string;
  microsoftEntraClientSecret: string;
  microsoftEntraTenant: string;
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
    authSecret: await readSecret("auth_secret"),
    jwtRs256PrivateKey: await readSecret("jwt_rs256_private_key"),
    jwtRs256PublicKey: await readSecret("jwt_rs256_public_key"),
    microsoftEntraClientId: await readSecret("microsoft_entra_client_id"),
    microsoftEntraClientSecret: await readSecret(
      "microsoft_entra_client_secret",
    ),
    microsoftEntraTenant: await readSecret("microsoft_entra_tenant"),
  };

  return secrets;
}

interface Config {
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
    publicUrl,
    apiUrl,
  };

  return appConfig;
}
