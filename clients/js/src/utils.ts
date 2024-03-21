export function toWords(str: string) {
  const camelCaseRegex = /([a-z0-9])([A-Z])/g;
  return str.replace(camelCaseRegex, '$1 $2');
}

export function capitalizeFirstLetter(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function lowercaseFirstLetter(str: string) {
  return str.charAt(0).toLowerCase() + str.slice(1);
}
