import { expect, test } from "bun:test";
import { subWhitespaceWithPercent } from "./query-utils";

test("whitespace", () => {
  expect(subWhitespaceWithPercent("let's have-falafel_for \t dinner\n")).toBe(
    "%let's%have%falafel%for%%%dinner%%",
  );
});
