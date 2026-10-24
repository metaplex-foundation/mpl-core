import {
  combineCodec,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/kit';
import {
  getBaseCollectionV1AccountDataDecoder,
  getBaseCollectionV1AccountDataEncoder,
  type BaseCollectionV1AccountDataArgs,
  type BaseCollectionV1AccountData,
} from '../generated/types/baseCollectionV1AccountData.js';
import { getPluginAccountDataEncoder } from './pluginAccountEncoder.js';
import { Key } from '../generated/types/key.js';
import {
  getPluginAccountDataDecoder,
  type PluginAccountData,
} from './pluginAccountData.js';

export type CollectionV1AccountData = BaseCollectionV1AccountData &
  PluginAccountData;

export function getCollectionV1AccountDataDecoder(): Decoder<CollectionV1AccountData> {
  return getPluginAccountDataDecoder(
    getBaseCollectionV1AccountDataDecoder(),
    getBaseCollectionV1AccountDataEncoder(),
    Key.CollectionV1
  );
}

export type CollectionV1AccountDataArgs = BaseCollectionV1AccountDataArgs &
  Partial<PluginAccountData>;

export function getCollectionV1AccountDataEncoder(): Encoder<CollectionV1AccountDataArgs> {
  return getPluginAccountDataEncoder(getBaseCollectionV1AccountDataEncoder());
}

export function getCollectionV1AccountDataCodec(): Codec<
  CollectionV1AccountDataArgs,
  CollectionV1AccountData
> {
  return combineCodec(
    getCollectionV1AccountDataEncoder(),
    getCollectionV1AccountDataDecoder()
  );
}
