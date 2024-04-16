import { PublicKey } from "@metaplex-foundation/umi";
import { BasePluginAuthority } from "../generated";

export type PluginAuthority = {
  type: PluginAuthorityType;
  address?: PublicKey;
};

export type PluginAuthorityType = BasePluginAuthority['__kind'];

export function pluginAuthorityToBase(u: PluginAuthority): BasePluginAuthority {
  if (u.type === 'Address' ) {
    return {
      __kind: 'Address',
      address: u.address as PublicKey,
    };
  }
  return {
    __kind: u.type,
  };
}