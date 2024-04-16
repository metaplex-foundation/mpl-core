import { PublicKey } from "@metaplex-foundation/umi";
import { BaseRoyalties, BaseRuleSet } from "../generated";
import { BasePlugin } from "./types";

export type RuleSet = ({
  type: 'None'
} | {
  type: 'ProgramAllowList';
  addresses: PublicKey[];
} | {
  type: 'ProgramDenyList';
  addresses: PublicKey[];
}); 

export type Royalties = Omit<BaseRoyalties, 'ruleSet'> & {
  ruleSet: RuleSet ;
}

export type RoyaltiesArgs = Royalties

export type RoyaltiesPlugin = BasePlugin & Royalties;

export function royaltiesToBase(r: Royalties): BaseRoyalties {
  let ruleSet: BaseRuleSet
  if (r.ruleSet.type === 'ProgramAllowList' || r.ruleSet.type === 'ProgramDenyList') {
    ruleSet = {
      __kind: r.ruleSet.type,
      fields: [r.ruleSet.addresses]
    }
  } else {
    ruleSet = { __kind: r.ruleSet.type }
  }
  return {
    ...r,
    ruleSet,
  }
}