import { Pool, type PoolConfig } from "pg";
import { readConfig } from "./config";

let dbClient: Pool | null = null;

export async function getDbClient() {
  if (dbClient !== null) {
    return dbClient;
  }

  const { dbHost, dbPassword, dbName, dbPort, maxDbPoolSize } =
    await readConfig();

  const options: PoolConfig = {
    user: "auth",
    password: dbPassword,
    host: dbHost,
    port: dbPort,
    database: dbName,
    max: maxDbPoolSize,
  };

  dbClient = new Pool(options);

  return dbClient;
}
