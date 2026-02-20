import { expect, test } from "bun:test";
import { Query } from "./query-utils";
import type { InstitutionFilter } from "cellnoor-client";

test("query serialization", () => {
  const q: Query<InstitutionFilter, string[]> = new Query({
    ids: ["i want", "falafel  for", "dinner- PLEASE"],
    names: [],
  });

  const parsedQueryString = JSON.parse(q.toQuerystring());

  expect(parsedQueryString.filter.ids).toStrictEqual([
    "%i_want%",
    "%falafel__for%",
    "%dinner__PLEASE%",
  ]);
});
