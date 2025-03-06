import { BaseMasterEdition } from '../generated';
import { someOrNone, unwrapOption } from '../utils';

export type MasterEdition = {
  maxSupply?: number;
  name?: string;
  uri?: string;
};

export type MasterEditionArgs = MasterEdition;

export function masterEditionToBase(s: MasterEdition): BaseMasterEdition {
  return {
    maxSupply: someOrNone(s.maxSupply),
    name: someOrNone(s.name),
    uri: someOrNone(s.uri),
  };
}

export function masterEditionFromBase(s: BaseMasterEdition): MasterEdition {
  return {
    maxSupply: unwrapOption(s.maxSupply),
    name: unwrapOption(s.name),
    uri: unwrapOption(s.uri),
  };
}
