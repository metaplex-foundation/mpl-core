import { none, Option, some } from '@metaplex-foundation/umi';

export type RenameField<T, K extends keyof T, R extends PropertyKey> = Omit<
  T,
  K
> &
  (undefined extends T[K] ? { [P in R]?: T[K] } : { [P in R]: T[K] });

export type RenameToType<T extends { __kind: string }> = T extends T
  ? RenameField<T, '__kind', 'type'>
  : never;

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

export function someOrNone<T>(value: T | undefined): Option<T> {
  return value !== undefined ? some(value) : none();
}

export function unwrapOption<T>(value: Option<T>): T | undefined {
  return value.__option === 'Some' ? value.value : undefined;
}
