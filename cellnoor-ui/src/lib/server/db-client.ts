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
    max: 2, // This app doesn't really need to connect to the db much, and even when it acts as a full-fledged auth-server, the operations should be very quick
  });

  return dbClient;
}
