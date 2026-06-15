const dateOptions = { day: "2-digit", month: "long", year: "numeric" };

// @ts-expect-error I hate this language
export const DATE_FORMATTER = Intl.DateTimeFormat("en-GB", dateOptions);

// @ts-expect-error I hate this language
export const DATETIME_FORMATTER = Intl.DateTimeFormat("en-GB", {
  ...dateOptions,
  hour: "numeric",
  hour12: true,
  minute: "2-digit",
});
