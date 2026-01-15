import type { MicrosoftEntraIDProfile } from "better-auth/social-providers";

export async function upsertPersonIntoDb(
  {
    name,
    email,
    email_verified,
    tid,
    oid,
  }: MicrosoftEntraIDProfile,
  dbClient: Bun.SQL,
): Promise<
  {
    id: string;
    is_admin: boolean;
    is_biology_staff: boolean;
    is_computational_staff: boolean;
  }
> {
  const newPerson = {
    name,
    email,
    email_verified: email_verified ?? false,
    institution_id: tid,
    microsoft_entra_oid: oid,
  };

  const createdPerson = await dbClient.begin(async (tx) => {
    // Anyone else with this email should have it removed
    await tx`update people set email = ${null}, email_verified = ${false} where email = ${newPerson.email}`;

    const result = await tx`insert into people ${
      tx(
        newPerson,
      )
    } on conflict (microsoft_entra_oid) do update set ${
      tx(
        newPerson,
      )
    } returning id, is_admin, is_biology_staff, is_computational_staff`;
    // Note that we don't need the user's name and email because better-auth already has that
    return result[0];
  });

  return createdPerson;
}

export async function getUserApiTokens(
  userId: string,
  dbClient: Bun.SQL,
): Promise<
  {
    jti: string;
    name: string;
    description: string;
    iat: Date;
    exp: Date;
  }[]
> {
  return await dbClient`select jti, name, description, iat, exp from api_tokens where sub = ${userId} order by iat`;
}

export async function insertApiToken(
  data: {
    jti: string;
    sub: string;
    name: string;
    description?: string | null;
    iat: Date;
    exp: Date;
  },
  dbClient: Bun.SQL,
) {
  await dbClient`insert into api_tokens ${dbClient(data)}`;
}
