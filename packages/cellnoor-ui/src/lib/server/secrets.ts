import { building } from "$app/env";
import { AUTH_SECRET } from "$app/env/private";

interface Secrets {
  authSecret: string;
}

let secrets: Secrets | null = null;

export async function readSecrets() {
  if (building) {
    return {
      authSecret: "",
    };
  }

  if (secrets !== null) {
    return secrets;
  }

  secrets = {
    authSecret: AUTH_SECRET || await readFromSecretFile("auth_secret"),
  };


  return secrets;
}

async function readFromSecretFile(name: string) {
  return await Bun.file(`/run/secrets/${name}`).text();
}
