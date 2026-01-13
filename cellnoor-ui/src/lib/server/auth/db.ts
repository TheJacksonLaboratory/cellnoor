import type { MicrosoftEntraIDProfile } from "better-auth/social-providers";
import { readConfig } from "../config";

export async function upsertPersonIntoDb(
  {
    name,
    email,
    emailVerified,
    tid,
    oid,
  }: {
    emailVerified: boolean;
  } & MicrosoftEntraIDProfile,
  dbClient: Bun.SQL,
): Promise<string> {
  const newPerson = {
    name,
    email,
    email_verified: emailVerified,
    institution_id: tid,
    microsoft_entra_oid: oid,
  };

  const newPersonId = await dbClient.begin(async (tx) => {
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
    } returning id`;
    const newPersonId = result[0].id;

    // Create a db user corresponding to this person so we can assign them roles later on. Note that we set a random
    // password and no roles so that nobody can log into the database as that user.
    await tx`select create_user_if_not_exists(${newPersonId}, ${
      crypto
        .getRandomValues(new Uint8Array(32))
        .toHex()
    }, '{}')`;

    return newPersonId;
  });

  return newPersonId;
}
