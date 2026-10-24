import {
  combineCodec,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/kit';
import {
  getBaseGroupV1AccountDataDecoder,
  getBaseGroupV1AccountDataEncoder,
  type BaseGroupV1AccountDataArgs,
  type BaseGroupV1AccountData,
} from '../generated/types/baseGroupV1AccountData.js';
import { getPluginAccountDataEncoder } from './pluginAccountEncoder.js';
import { Key } from '../generated/types/key.js';
import {
  getPluginAccountDataDecoder,
  type PluginAccountData,
} from './pluginAccountData.js';

export type GroupV1AccountData = BaseGroupV1AccountData & PluginAccountData;

export function getGroupV1AccountDataDecoder(): Decoder<GroupV1AccountData> {
  return getPluginAccountDataDecoder(
    getBaseGroupV1AccountDataDecoder(),
    getBaseGroupV1AccountDataEncoder(),
    Key.GroupV1
  );
}

export type GroupV1AccountDataArgs = BaseGroupV1AccountDataArgs &
  Partial<PluginAccountData>;

export function getGroupV1AccountDataEncoder(): Encoder<GroupV1AccountDataArgs> {
  return getPluginAccountDataEncoder(getBaseGroupV1AccountDataEncoder());
}

export function getGroupV1AccountDataCodec(): Codec<
  GroupV1AccountDataArgs,
  GroupV1AccountData
> {
  return combineCodec(
    getGroupV1AccountDataEncoder(),
    getGroupV1AccountDataDecoder()
  );
}
