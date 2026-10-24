import { type Address } from '@solana/addresses';
import { type BaseRoyalties, type BaseRuleSet } from '../generated';
import { type BasePlugin } from './types';

export type UnwrappedRuleSet =
  | {
      type: 'None';
    }
  | {
      type: 'ProgramAllowList';
      addresses: Address[];
    }
  | {
      type: 'ProgramDenyList';
      addresses: Address[];
    };

export type RuleSet = UnwrappedRuleSet | BaseRuleSet;

export type Royalties = Omit<BaseRoyalties, 'ruleSet'> & {
  ruleSet: RuleSet;
};

export type RoyaltiesArgs = Royalties;

export type RoyaltiesPlugin = BasePlugin & Royalties;

export function ruleSetToBase(r: RuleSet): BaseRuleSet {
  const base = r as BaseRuleSet;
  if (base.__kind) {
    return r as BaseRuleSet;
  }
  const ruleSet = r as UnwrappedRuleSet;

  if (
    ruleSet.type === 'ProgramAllowList' ||
    ruleSet.type === 'ProgramDenyList'
  ) {
    return {
      __kind: ruleSet.type,
      fields: [ruleSet.addresses],
    };
  }
  return { __kind: ruleSet.type };
}

export function royaltiesToBase(r: Royalties): BaseRoyalties {
  return {
    ...r,
    ruleSet: ruleSetToBase(r.ruleSet),
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
      type: r.ruleSet.__kind,
    };
  }
  return {
    ...r,
    ruleSet,
  };
}
