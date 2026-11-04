// This module could be improved but I hate writing TypeScript so it's not worth it
function readEnvVar(name: string) {
  const key = name.toUpperCase();
  const val = Bun.env[`CELLNOOR_AUTH__${key}`];

  return val;
}

function readRequiredEnvVar(name: string) {
  const val = readEnvVar(name);

  if (val === undefined) {
    throw `required environment variable ${name} is unset`;
  }

  return val;
}

function readBooleanEnvVar(name: string) {
  const val = readEnvVar(name);

  if (val === undefined || val === "false") {
    return false;
  }

  if (val === "true") {
    return true;
  }

  throw `environment variable ${name} must be "true" or "false", but it is "${val}"`;
}

async function readSecret(name: string): Promise<string> {
  if (Bun.env.CELLNOOR_AUTH__READ_SECRET_FILE) {
    return await Bun.file(`/run/secrets/${name}`).text();
  }

  return readRequiredEnvVar(name);
}

interface Config {
  publicBaseUrl: string;
  unixDomainSocket?: string;
  dbPassword: string;
  dbHost: string;
  dbPort?: number;
  dbName?: string;
  maxDbPoolSize?: number;
  authSecret: string;
  microsoftEntraCommonTenant: boolean;
  microsoftEntraClientId: string;
  microsoftEntraClientSecret: string;
}

let appConfig: Config | null = null;

export async function readConfig(): Promise<Config> {
  if (appConfig !== null) {
    return appConfig;
  }

  const maxDbPoolSizeFromEnv = readEnvVar("max_db_pool_size");
  let maxDbPoolSize = undefined;
  if (maxDbPoolSizeFromEnv) {
    maxDbPoolSize = Number.parseInt(maxDbPoolSizeFromEnv);
  }

  const dbPortFromEnv = readEnvVar("db_port");
  let dbPort = undefined;
  if (dbPortFromEnv) {
    dbPort = Number.parseInt(dbPortFromEnv);
  }

  appConfig = {
    publicBaseUrl: `https://${readRequiredEnvVar("public_base_url")}`,
    unixDomainSocket: readEnvVar("unix_domain_socket"),
    // These 3 database-related things have double underscores to match how cellnoor-api reads environment variables
    dbPassword: await readSecret("db__password"),
    dbHost: readRequiredEnvVar("db__host"),
    dbPort,
    dbName: readRequiredEnvVar("db__name"),
    maxDbPoolSize,
    authSecret: await readSecret("auth_secret"),
    microsoftEntraCommonTenant: readBooleanEnvVar(
      "microsoft_entra_common_tenant",
    ),
    microsoftEntraClientId: await readSecret("microsoft_entra_client_id"),
    microsoftEntraClientSecret: await readSecret(
      "microsoft_entra_client_secret",
    ),
  };

  return appConfig;
}
