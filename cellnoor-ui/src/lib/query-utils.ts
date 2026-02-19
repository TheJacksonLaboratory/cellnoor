const whitespaceRegex = /\s|_|-/g;

export function subWhitespaceWithPercent(s: string) {
  return `%${s.replace(whitespaceRegex, "%")}%`;
}
