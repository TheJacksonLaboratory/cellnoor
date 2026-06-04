import { expect, test } from "bun:test";
import { type Query, toQueryString } from "./query-utils.svelte";
import type { InstitutionFilter } from "cellnoor-client";

test("query serialization", () => {
  const q: Query<InstitutionFilter, string[]> = {
    filter: {
      ids: ["i want", "falafel  for", "dinner- PLEASE"],
      names: [],
    },
    limit: 0,
  };

  const parsedQueryString = JSON.parse(toQueryString(q));

  expect(parsedQueryString.filter.ids).toStrictEqual([
    "%i_want%",
    "%falafel__for%",
    "%dinner__PLEASE%",
  ]);
});
