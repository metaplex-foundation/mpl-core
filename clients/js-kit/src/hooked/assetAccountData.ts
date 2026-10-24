import {
  combineCodec,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/kit';
import {
  getBaseAssetV1AccountDataDecoder,
  getBaseAssetV1AccountDataEncoder,
  type BaseAssetV1AccountDataArgs,
  type BaseAssetV1AccountData,
} from '../generated/types/baseAssetV1AccountData.js';
import { getPluginAccountDataEncoder } from './pluginAccountEncoder.js';
import { Key } from '../generated/types/key.js';
import {
  getPluginAccountDataDecoder,
  type PluginAccountData,
} from './pluginAccountData.js';

export type AssetV1AccountData = BaseAssetV1AccountData & PluginAccountData;

export function getAssetV1AccountDataDecoder(): Decoder<AssetV1AccountData> {
  return getPluginAccountDataDecoder(
    getBaseAssetV1AccountDataDecoder(),
    getBaseAssetV1AccountDataEncoder(),
    Key.AssetV1
  );
}

export type AssetV1AccountDataArgs = BaseAssetV1AccountDataArgs &
  Partial<PluginAccountData>;

export function getAssetV1AccountDataEncoder(): Encoder<AssetV1AccountDataArgs> {
  return getPluginAccountDataEncoder(getBaseAssetV1AccountDataEncoder());
}

export function getAssetV1AccountDataCodec(): Codec<
  AssetV1AccountDataArgs,
  AssetV1AccountData
> {
  return combineCodec(
    getAssetV1AccountDataEncoder(),
    getAssetV1AccountDataDecoder()
  );
}
