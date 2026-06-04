import { readSecrets } from "$lib/server/config";

let dbClient: Bun.SQL | null = null;

export async function getDbClient() {
  if (dbClient !== null) {
    return dbClient;
  }

  const { cellnoorUiDbPassword, dbHost, dbPort, dbName } = await readSecrets();

  const options: Bun.SQL.Options = {
    username: "cellnoor_ui",
    password: cellnoorUiDbPassword,
    port: dbPort,
    database: dbName,
    max: 2,
  };

  if (dbHost.startsWith("/")) {
    options.path = dbHost;
  } else {
    options.hostname = dbHost;
  }

  dbClient = new Bun.SQL(options);

  return dbClient;
}
