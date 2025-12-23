import { readSecrets } from "$lib/server/config";

let dbClient: Bun.SQL | null = null;

export async function getDbClient() {
  if (dbClient !== null) {
    return dbClient;
  }

  const secrets = await readSecrets();
  dbClient = new Bun.SQL({
    username: "cellnoor_ui",
    password: secrets.cellnoorUiDbPassword,
    hostname: secrets.dbHost,
    port: secrets.dbPort,
    database: secrets.dbName,
  });

  return dbClient;
}
