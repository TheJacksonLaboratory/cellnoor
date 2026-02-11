import { readSecrets } from "$lib/server/config";

let dbClient: Bun.SQL | null = null;

export async function getDbClient() {
  if (dbClient !== null) {
    return dbClient;
  }

  const { cellnoorUiDbPassword, dbHost, dbPort, dbName } = await readSecrets();
  dbClient = new Bun.SQL({
    username: "cellnoor_ui",
    password: cellnoorUiDbPassword,
    hostname: dbHost,
    port: dbPort,
    database: dbName,
  });

  return dbClient;
}
