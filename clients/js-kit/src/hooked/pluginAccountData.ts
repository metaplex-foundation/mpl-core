import { decode as decodeMessagePack } from '@msgpack/msgpack';
import {
  createDecoder,
  getEncodedSize,
  isSome,
  type Decoder,
  type Encoder,
  type ReadonlyUint8Array,
} from '@solana/kit';
import {
  getPluginHeaderV1Decoder,
  type PluginHeaderV1,
} from '../generated/accounts/pluginHeaderV1.js';
import {
  getPluginRegistryV1Decoder,
  getPluginRegistryV1Encoder,
} from '../generated/accounts/pluginRegistryV1.js';
import {
  getExternalPluginAdapterDecoder,
  getExternalPluginAdapterEncoder,
  type ExternalPluginAdapter,
} from '../generated/types/externalPluginAdapter.js';
import { ExternalPluginAdapterSchema } from '../generated/types/externalPluginAdapterSchema.js';
import { ExternalPluginAdapterType } from '../generated/types/externalPluginAdapterType.js';
import type { ExternalRegistryRecord } from '../generated/types/externalRegistryRecord.js';
import { Key } from '../generated/types/key.js';
import {
  getPluginDecoder,
  getPluginEncoder,
  type Plugin,
} from '../generated/types/plugin.js';
import { PluginType } from '../generated/types/pluginType.js';
import type { RegistryRecord } from '../generated/types/registryRecord.js';

export type DecodedPlugin = RegistryRecord & { plugin: Plugin };

export type DecodedExternalPluginAdapter = ExternalRegistryRecord & {
  adapter: ExternalPluginAdapter;
  data?: unknown;
  dataError?: unknown;
  rawData?: ReadonlyUint8Array;
};

export type PluginAccountData = {
  externalPluginAdapters: DecodedExternalPluginAdapter[];
  pluginHeader?: PluginHeaderV1;
  plugins: DecodedPlugin[];
};

function checkedOffset(
  value: bigint,
  minimum: number,
  maximum: number,
  label: string
): number {
  if (value < BigInt(minimum) || value > BigInt(maximum)) {
    throw new Error(`Invalid ${label}: out of bounds.`);
  }
  return Number(value);
}

function readValidated<T>(
  decoder: Decoder<T>,
  encoder: Encoder<T>,
  bytes: ReadonlyUint8Array,
  offset = 0
): [T, number] {
  const [value, end] = decoder.read(bytes, offset);
  if (end - offset !== getEncodedSize(value, encoder)) {
    throw new Error('Invalid account data: truncated encoding.');
  }
  return [value, end];
}

/** Decode the base account and the payloads addressed by its plugin registry. */
export function getPluginAccountDataDecoder<T extends { key: Key }>(
  baseDecoder: Decoder<T>,
  baseEncoder: Encoder<T>,
  expectedKey: Key
): Decoder<T & PluginAccountData> {
  return createDecoder<T & PluginAccountData>({
    read(bytes, offset) {
      // Registry offsets are relative to the account, not to the enclosing buffer.
      const account = bytes.subarray(offset);
      const [base, baseEnd] = readValidated(baseDecoder, baseEncoder, account);
      if (base.key !== expectedKey) {
        throw new Error(
          `Invalid account key: expected ${expectedKey}, got ${base.key}.`
        );
      }
      if (baseEnd === account.length) {
        return [
          { ...base, externalPluginAdapters: [], plugins: [] },
          offset + baseEnd,
        ];
      }

      const [pluginHeader, headerEnd] = getPluginHeaderV1Decoder().read(
        account,
        baseEnd
      );
      if (pluginHeader.key !== Key.PluginHeaderV1) {
        throw new Error('Invalid plugin header key.');
      }
      const registryOffset = checkedOffset(
        pluginHeader.pluginRegistryOffset,
        headerEnd,
        account.length - 1,
        'plugin registry offset'
      );
      const [registry, registryEnd] = readValidated(
        getPluginRegistryV1Decoder(),
        getPluginRegistryV1Encoder(),
        account,
        registryOffset
      );
      if (registry.key !== Key.PluginRegistryV1) {
        throw new Error('Invalid plugin registry key.');
      }
      if (registryEnd > account.length) {
        throw new Error('Invalid plugin registry: truncated data.');
      }

      const pluginOffsets = [
        ...registry.registry,
        ...registry.externalRegistry,
      ].map((record) =>
        checkedOffset(
          record.offset,
          headerEnd,
          registryOffset - 1,
          'plugin offset'
        )
      );
      if (new Set(pluginOffsets).size !== pluginOffsets.length) {
        throw new Error('Invalid plugin offsets: duplicate payloads.');
      }
      const dataRanges = registry.externalRegistry.map((record) => {
        if (isSome(record.dataOffset) !== isSome(record.dataLen)) {
          throw new Error(
            'Invalid external plugin data range: missing offset or length.'
          );
        }
        if (!isSome(record.dataOffset) || !isSome(record.dataLen))
          return undefined;
        const start = checkedOffset(
          record.dataOffset.value,
          headerEnd,
          registryOffset,
          'external plugin data range'
        );
        const end = checkedOffset(
          record.dataOffset.value + record.dataLen.value,
          start,
          registryOffset,
          'external plugin data range'
        );
        return { end, start };
      });
      const starts = [
        ...pluginOffsets,
        ...dataRanges.flatMap((range) => (range ? [range.start] : [])),
        registryOffset,
      ].sort((a, b) => a - b);
      const endOfPayload = (start: number) =>
        starts.find((value) => value > start) ?? registryOffset;
      dataRanges.forEach((range) => {
        if (
          range &&
          (range.end > endOfPayload(range.start) ||
            (range.end > range.start && pluginOffsets.includes(range.start)))
        ) {
          throw new Error(
            'Invalid external plugin data range: overlapping payloads.'
          );
        }
      });

      const plugins = registry.registry.map((record): DecodedPlugin => {
        const start = Number(record.offset);
        const [plugin] = readValidated(
          getPluginDecoder(),
          getPluginEncoder(),
          account.subarray(start, endOfPayload(start))
        );
        if (PluginType[record.pluginType] !== plugin.__kind) {
          throw new Error(
            'Invalid plugin discriminant: registry and payload differ.'
          );
        }
        return { ...record, plugin };
      });
      const externalPluginAdapters = registry.externalRegistry.map(
        (record, index): DecodedExternalPluginAdapter => {
          const start = Number(record.offset);
          const [adapter, adapterSize] = readValidated(
            getExternalPluginAdapterDecoder(),
            getExternalPluginAdapterEncoder(),
            account.subarray(start, endOfPayload(start))
          );
          if (ExternalPluginAdapterType[record.pluginType] !== adapter.__kind) {
            throw new Error(
              'Invalid external plugin discriminant: registry and payload differ.'
            );
          }
          const range = dataRanges[index];
          if (!range) return { ...record, adapter };
          if (range.start < start + adapterSize) {
            throw new Error(
              'Invalid external plugin data range: overlaps its adapter.'
            );
          }
          const rawData = account.slice(range.start, range.end);
          const config = adapter.fields[0];
          let data: unknown = rawData;
          try {
            if ('schema' in config) {
              if (config.schema === ExternalPluginAdapterSchema.Json) {
                data = rawData.length
                  ? JSON.parse(
                      new TextDecoder('utf-8', { fatal: true }).decode(rawData)
                    )
                  : undefined;
              } else if (
                config.schema === ExternalPluginAdapterSchema.MsgPack
              ) {
                data = rawData.length ? decodeMessagePack(rawData) : null;
              }
            }
          } catch (dataError) {
            return { ...record, adapter, data: undefined, dataError, rawData };
          }
          return { ...record, adapter, data, rawData };
        }
      );
      return [
        { ...base, externalPluginAdapters, pluginHeader, plugins },
        offset + registryEnd,
      ];
    },
  });
}
