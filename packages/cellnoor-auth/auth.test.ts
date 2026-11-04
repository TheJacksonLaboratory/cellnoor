import { afterAll, beforeAll, describe, expect, spyOn, test } from "bun:test";
import { exportJWK, generateKeyPair, jwtVerify, SignJWT } from "jose";
import { Pool } from "pg";
import { auth } from "./auth";
import { readConfig } from "./config";
import { getDbClient } from "./db";
import type { MicrosoftEntraIDProfile } from "better-auth";

const {
  publicBaseUrl,
  authSecret,
  dbHost,
  dbPort,
  dbName,
  microsoftEntraClientId: clientId,
} = await readConfig();

type PartialMicrosoftEntraIDProfile = Pick<
  MicrosoftEntraIDProfile,
  "tid" | "email" | "oid" | "name"
> & { email: string };

// The `auth` role can't insert institutions, so fixtures are written as the superuser
const db = new Pool({
  user: "postgres",
  password: Bun.env.CELLNOOR_DB_ROOT_PASSWORD,
  host: dbHost,
  port: dbPort,
  database: dbName,
});

const entraEndpoint = "https://login.microsoftonline.com/organizations";
const signingKey = await generateKeyPair("RS256");
const signingJwk = {
  ...(await exportJWK(signingKey.publicKey)),
  kid: "cellnoor-auth-test",
  alg: "RS256",
};

function idToken({ tid, oid, email, name }: PartialMicrosoftEntraIDProfile) {
  return new SignJWT({
    tid,
    oid,
    email,
    name,
    preferred_username: email,
  })
    .setProtectedHeader({ alg: "RS256", kid: signingJwk.kid })
    .setIssuer(`https://login.microsoftonline.com/${tid}/v2.0`)
    .setAudience(clientId)
    .setSubject(crypto.randomUUID())
    .setIssuedAt()
    .setExpirationTime("1h")
    .sign(signingKey.privateKey);
}

// The fake Entra reads the identity from the authorization code that `signIn` sends
spyOn(globalThis, "fetch").mockImplementation((async (input, init) => {
  const request =
    input instanceof Request ? input : new Request(input.toString(), init);

  if (request.url === `${entraEndpoint}/oauth2/v2.0/token`) {
    const code = new URLSearchParams(await request.text()).get("code");

    return Response.json({
      access_token: "fake-access-token",
      token_type: "Bearer",
      expires_in: 3600,
      id_token: await idToken(JSON.parse(code!)),
    });
  }

  if (request.url === `${entraEndpoint}/discovery/v2.0/keys`) {
    return Response.json({ keys: [signingJwk] });
  }

  throw new Error(`unexpected request to ${request.url}`);
}) as typeof fetch);

type CookieJar = Map<string, string>;

function storeCookies(jar: CookieJar, response: Response) {
  for (const header of response.headers.getSetCookie()) {
    const cookie = Bun.Cookie.parse(header);

    if (cookie.isExpired() || cookie.value === "") {
      jar.delete(cookie.name);
    } else {
      jar.set(cookie.name, cookie.value);
    }
  }
}

function cookieHeader(jar: CookieJar) {
  return [...jar].map(([name, value]) => `${name}=${value}`).join("; ");
}

async function signIn(identity: PartialMicrosoftEntraIDProfile) {
  const cookies: CookieJar = new Map();

  const start = await auth.api.signInSocial({
    body: { provider: "microsoft", callbackURL: "/" },
    asResponse: true,
  });
  storeCookies(cookies, start);
  const { url } = (await start.json()) as { url: string };

  const callbackUrl = new URL(`${publicBaseUrl}/api/auth/callback/microsoft`);
  callbackUrl.searchParams.set("code", JSON.stringify(identity));
  callbackUrl.searchParams.set(
    "state",
    new URL(url).searchParams.get("state")!,
  );

  const callback = await auth.handler(
    new Request(callbackUrl.href, {
      headers: { cookie: cookieHeader(cookies) },
    }),
  );
  expect(callback.status).toBe(302);
  storeCookies(cookies, callback);

  const location = new URL(callback.headers.get("location")!, publicBaseUrl);

  return { error: location.searchParams.get("error"), cookies };
}

// Mirrors `read_chunked_jwt` in crates/cellnoor-api/src/auth.rs
function sessionJwt(cookies: CookieJar) {
  const chunkNumber = (name: string) => {
    const n = Number(name.slice(name.lastIndexOf(".") + 1));
    return Number.isInteger(n) ? n : -1;
  };

  return [...cookies]
    .filter(
      ([name]) =>
        name.startsWith("__Secure-cellnoor-auth.session_data") ||
        name.startsWith("cellnoor-auth.session_data"),
    )
    .sort(([a], [b]) => chunkNumber(a) - chunkNumber(b))
    .map(([, value]) => value)
    .join("");
}

const institutionTenantId = crypto.randomUUID();
const otherInstitutionTenantId = crypto.randomUUID();

function newIdentity(
  tid = institutionTenantId,
): PartialMicrosoftEntraIDProfile {
  const oid = crypto.randomUUID();
  return { tid, oid, email: `${oid}@cellnoor-auth.test`, name: "Test Person" };
}

async function insertInstitution(tenant: string): Promise<string> {
  const {
    rows: [{ id }],
  } = await db.query(
    "insert into institution (name, microsoft_entra_tenant_id) values ($1, $2) returning id",
    [`cellnoor-auth test ${tenant}`, tenant],
  );
  return id;
}

let institutionId: string;
let otherInstitutionId: string;

// Mirrors what crates/cellnoor-api/src/handlers/people/create.rs writes
async function insertPerson({
  email = null,
  emailVerified,
  isStaff = false,
  oid,
}: {
  email?: string | null;
  emailVerified?: boolean;
  isStaff?: boolean;
  oid?: string;
}): Promise<string> {
  const {
    rows: [{ id }],
  } = await db.query(
    "insert into person (name, email, institution_id) values ($1, $2, $3) returning id",
    ["Test Person", email, institutionId],
  );

  if (emailVerified !== undefined) {
    await db.query("update person set email_verified = $1 where id = $2", [
      emailVerified,
      id,
    ]);
  }

  if (isStaff) {
    await db.query("update principal set is_staff = true where id = $1", [id]);
  }

  if (oid) {
    await db.query(
      "insert into account (person_id, auth_provider, auth_provider_user_id) values ($1, 'microsoft', $2)",
      [id, oid],
    );
  }

  return id;
}

async function signInPreProvisionedPerson({ isStaff = false } = {}) {
  const identity = newIdentity();
  const personId = await insertPerson({ oid: identity.oid, isStaff });

  const { error, cookies } = await signIn(identity);
  expect(error).toBeNull();

  return { personId, cookies };
}

async function personIdsWithEmail(email: string): Promise<string[]> {
  const { rows } = await db.query("select id from person where email = $1", [
    email,
  ]);
  return rows.map(({ id }) => id);
}

async function accountOwners(oid: string): Promise<string[]> {
  const { rows } = await db.query(
    "select person_id from account where auth_provider = 'microsoft' and auth_provider_user_id = $1",
    [oid],
  );
  return rows.map(({ person_id }) => person_id);
}

beforeAll(async () => {
  institutionId = await insertInstitution(institutionTenantId);
  otherInstitutionId = await insertInstitution(otherInstitutionTenantId);
});

afterAll(async () => {
  const institutions = [institutionId, otherInstitutionId];
  await db.query("delete from person where institution_id = any($1)", [
    institutions,
  ]);
  await db.query("delete from institution where id = any($1)", [institutions]);

  await db.end();
  await (await getDbClient()).end();
});

describe("signing in with Microsoft", () => {
  test("does not link a pre-existing person whose email is unverified", async () => {
    const identity = newIdentity();
    const personId = await insertPerson({
      email: identity.email,
      emailVerified: false,
    });

    const { error } = await signIn(identity);

    expect(error).toBe("account_not_linked");
    expect(await accountOwners(identity.oid)).toEqual([]);
    expect(await personIdsWithEmail(identity.email)).toEqual([personId]);
  });

  test("links a pre-existing person whose email is verified", async () => {
    const identity = newIdentity();
    const personId = await insertPerson({
      email: identity.email,
      emailVerified: true,
    });

    const { error } = await signIn(identity);

    expect(error).toBeNull();
    expect(await accountOwners(identity.oid)).toEqual([personId]);
    expect(await personIdsWithEmail(identity.email)).toEqual([personId]);
  });

  test("gives the next holder of a reassigned email a new person", async () => {
    const original = newIdentity();
    const personId = await insertPerson({ email: original.email });
    expect((await signIn(original)).error).toBeNull();

    const nextHolderOfEmail = { ...newIdentity(), email: original.email };
    const { error } = await signIn(nextHolderOfEmail);

    expect(error).toBeNull();
    const newPersonIds = await accountOwners(nextHolderOfEmail.oid);
    expect(newPersonIds).toHaveLength(1);
    expect(newPersonIds).not.toContain(personId);
    expect(await personIdsWithEmail(original.email)).toEqual(newPersonIds);
    expect(await accountOwners(original.oid)).toEqual([personId]);
  });

  test("signs in a person with a pre-provisioned account and sets their email", async () => {
    const identity = newIdentity();
    const personId = await insertPerson({ oid: identity.oid });

    const { error } = await signIn(identity);

    expect(error).toBeNull();
    expect(await accountOwners(identity.oid)).toEqual([personId]);
    expect(await personIdsWithEmail(identity.email)).toEqual([personId]);
  });

  test("replaces the email of a person whose account matches", async () => {
    const identity = newIdentity();
    const oldEmail = `old-${identity.email}`;
    const personId = await insertPerson({ email: oldEmail, oid: identity.oid });

    const { error } = await signIn(identity);

    expect(error).toBeNull();
    expect(await personIdsWithEmail(identity.email)).toEqual([personId]);
    expect(await personIdsWithEmail(oldEmail)).toEqual([]);
  });

  test("creates an unknown person in the institution that owns their tenant", async () => {
    const identity = newIdentity(otherInstitutionTenantId);

    const { error } = await signIn(identity);

    expect(error).toBeNull();
    const { rows } = await db.query(
      "select id, institution_id from person where email = $1",
      [identity.email],
    );
    expect(rows).toEqual([
      { id: expect.any(String), institution_id: otherInstitutionId },
    ]);
    expect(await accountOwners(identity.oid)).toEqual([rows[0].id]);
  });
});

describe("the session cookie read by cellnoor-api", () => {
  test.each([false, true])(
    "is an HS256 JWT carrying the person's id and is_staff = %p",
    async (isStaff) => {
      const { personId, cookies } = await signInPreProvisionedPerson({
        isStaff,
      });

      const { payload } = await jwtVerify(
        sessionJwt(cookies),
        new TextEncoder().encode(authSecret),
        {
          algorithms: ["HS256"],
        },
      );

      expect(payload).toMatchObject({
        user: { id: personId, is_staff: isStaff },
        iat: expect.any(Number),
        exp: expect.any(Number),
      });
    },
  );
});

describe("HTTP endpoints", () => {
  const enabledPaths = [
    "/sign-in/social",
    "/callback/:id",
    "/get-session",
    "/sign-out",
    "/error",
  ];
  // `disabledPaths` compares exact request paths, so it can't disable a path with a parameter
  const parameterizedPaths = ["/reset-password/:token"];

  const disabledEndpoints = Object.values(auth.api).filter(
    ({ path }) =>
      !enabledPaths.includes(path) && !parameterizedPaths.includes(path),
  );

  test.each(disabledEndpoints)(
    "$path is disabled",
    async ({ path, options }) => {
      for (const method of [options.method].flat()) {
        const response = await auth.handler(
          new Request(`${publicBaseUrl}/api/auth${path}`, {
            method,
            headers: { origin: publicBaseUrl },
          }),
        );

        expect(response.status).toBe(404);
      }
    },
  );
});

describe("/update-user", () => {
  async function updateUser(cookies: CookieJar, body: object) {
    return auth.handler(
      new Request(`${publicBaseUrl}/api/auth/update-user`, {
        method: "POST",
        headers: {
          cookie: cookieHeader(cookies),
          origin: publicBaseUrl,
          "content-type": "application/json",
        },
        body: JSON.stringify(body),
      }),
    );
  }

  test("cannot change a person's institution", async () => {
    const { personId, cookies } = await signInPreProvisionedPerson();

    const response = await updateUser(cookies, {
      institution_id: otherInstitutionId,
    });

    expect(response.status).not.toBe(401);
    const { rows } = await db.query(
      "select institution_id from person where id = $1",
      [personId],
    );
    expect(rows).toEqual([{ institution_id: institutionId }]);
  });

  test("cannot make a person staff", async () => {
    const { personId, cookies } = await signInPreProvisionedPerson();

    const response = await updateUser(cookies, { is_staff: true });

    expect(response.status).not.toBe(401);
    const { rows } = await db.query(
      "select is_staff from principal where id = $1",
      [personId],
    );
    expect(rows).toEqual([{ is_staff: false }]);
  });
});
