// This module could be improved but I hate writing TypeScript so it's not worth it
function readEnvVar(name: string) {
  const key = name.toUpperCase();
  const val = Bun.env[key] || Bun.env[`CELLNOOR_AUTH__${key}`] || Bun.env[`CELLNOOR__${key}`]

  return val;
}

function readRequiredEnvVar(name: string) {
  const val = readEnvVar(name);

  if (val === undefined) {
    throw `required environment variable ${name} is unset`;
  }

  return val;
}

async function readSecret(name: string): Promise<string> {
  if (Bun.env.READ_SECRET_FILE) {
    return await Bun.file(`/run/secrets/${name}`).text();
  }

  return readRequiredEnvVar(name);
}

interface Config {
  publicAuthUrl: string;
  unixDomainSocket?: string;
  dbPassword: string;
  dbHost: string;
  dbPort?: number;
  dbName?: string;
  maxDbPoolSize?: number;
  authSecret: string;
  microsoftEntraTenantId: string;
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

  return {
    publicAuthUrl: readRequiredEnvVar("public_auth_url"),
    unixDomainSocket: readEnvVar("unix_domain_socket"),
    dbPassword: await readSecret("auth_db_password"),
    dbHost: readRequiredEnvVar("db_host"),
    dbPort,
    dbName: readRequiredEnvVar("db_name"),
    maxDbPoolSize,
    microsoftEntraTenantId: await readSecret("microsoft_entra_tenant_id"),
    authSecret: await readSecret("auth_secret"),
    microsoftEntraClientId: await readSecret("microsoft_entra_client_id"),
    microsoftEntraClientSecret: await readSecret(
      "microsoft_entra_client_secret",
    ),
  };
}
