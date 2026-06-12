export function isNonempty(obj: Record<string, Record<string, Record<string, unknown[]>>>) {
  console.log(obj);
  const field = Object.values(obj)[0];
  if (!field) {
    return false;
  }

  console.log(field);


  const operator = Object.values(field)[0];
  if (!operator) {
    return false;
  }

  console.log(operator);

  const array = Object.values(operator)[0];
  if (!array) {
    return false;
  }

  console.log(array);

  return array.length > 0;
}
