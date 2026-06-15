export function isNonempty(obj: Record<string, Record<string, Record<string, unknown[]>>>) {
  const field = Object.values(obj)[0];
  if (!field) {
    return false;
  }


  const operator = Object.values(field)[0];
  if (!operator) {
    return false;
  }

  const array = Object.values(operator)[0];
  if (!array) {
    return false;
  }

  return array.length > 0;
}
