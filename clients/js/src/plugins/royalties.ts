import { PublicKey } from '@metaplex-foundation/umi';
import { BaseRoyalties, BaseRuleSet } from '../generated';
import { BasePlugin } from './types';

export type RuleSet =
  | {
      type: 'None';
      __kind?: 'None';
    }
  | {
      type: 'ProgramAllowList';
      // TODO allow this kind of backwards compatibility?
      __kind?: 'ProgramAllowList';
      addresses: PublicKey[];
      fields?: [Array<PublicKey>];
    }
  | {
      type: 'ProgramDenyList';
      __kind?: 'ProgramDenyList';
      addresses: PublicKey[];
      fields?: [Array<PublicKey>];
    };

export type Royalties = Omit<BaseRoyalties, 'ruleSet'> & {
  ruleSet: RuleSet;
};

export type RoyaltiesArgs = Royalties;

export type RoyaltiesPlugin = BasePlugin & Royalties;

export function royaltiesToBase(r: Royalties): BaseRoyalties {
  let ruleSet: BaseRuleSet;
  if (
    r.ruleSet.type === 'ProgramAllowList' ||
    r.ruleSet.type === 'ProgramDenyList'
  ) {
    ruleSet = {
      __kind: r.ruleSet.type,
      fields: [r.ruleSet.addresses],
    };
  } else {
    ruleSet = { __kind: r.ruleSet.type };
  }
  return {
    ...r,
    ruleSet,
  };
}

export function royaltiesFromBase(r: BaseRoyalties): Royalties {
  let ruleSet: RuleSet;
  if (r.ruleSet.__kind === 'ProgramAllowList') {
    ruleSet = {
      ...r.ruleSet,
      type: 'ProgramAllowList',
      addresses: r.ruleSet.fields[0],
    };
  } else if (r.ruleSet.__kind === 'ProgramDenyList') {
    ruleSet = {
      ...r.ruleSet,
      type: 'ProgramDenyList',
      addresses: r.ruleSet.fields[0],
    };
  } else {
    ruleSet = {
      ...r.ruleSet,
      type: r.ruleSet.__kind
    };
  }
  return {
    ...r,
    ruleSet,
  };
}