import { PublicKey } from '@metaplex-foundation/umi';
import { pluginAuthority } from './plugins';

/**
 * @deprecated use SDK v1 methods like `create` or `update` instead of `createV1` or `updateV1`. The new methods no longer require this helper
 *
 * @returns umi plugin authority with type 'None'
 */
export function nonePluginAuthority() {
  return pluginAuthority('None');
}

/**
 * @deprecated use SDK v1 methods like `create` or `update` instead of `createV1` or `updateV1`. The new methods no longer require this helper
 * @returns umi plugin authority with type 'None'
 */
export function ownerPluginAuthority() {
  return pluginAuthority('Owner');
}

/**
 * @deprecated use SDK v1 methods like `create` or `update` instead of `createV1` or `updateV1`. The new methods no longer require this helper
 * @returns umi plugin authority with type 'UpdateAuthority'
 */
export function updatePluginAuthority() {
  return pluginAuthority('UpdateAuthority');
}

/**
 * @deprecated use SDK v1 methods like `create` or `update` instead of `createV1` or `updateV1`. The new methods no longer require this helper
 * @returns umi plugin authority with type 'Address'
 */
export function addressPluginAuthority(address: PublicKey) {
  return pluginAuthority('Address', { address });
}
