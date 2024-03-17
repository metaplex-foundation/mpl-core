export function toWords(str: string) {
  const camelCaseRegex = /([a-z0-9])([A-Z])/g;
  return str.replace(camelCaseRegex, '$1 $2');
}
