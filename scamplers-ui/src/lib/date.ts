// They format their dates the right way in Canada
export const DATE_FORMATTER = Intl.DateTimeFormat("en-CA");

export const DATETIME_FORMATTER = Intl.DateTimeFormat(
  "en-CA",
  { dateStyle: "short", timeStyle: "short" },
);
