import {
  getBytesEncoder,
  none,
  some,
  transformEncoder,
  type Encoder,
  type ReadonlyUint8Array,
} from '@solana/kit';
import {
  getPluginHeaderV1Encoder,
  getPluginHeaderV1Size,
} from '../generated/accounts/pluginHeaderV1.js';
import { getPluginRegistryV1Encoder } from '../generated/accounts/pluginRegistryV1.js';
import { getExternalPluginAdapterEncoder } from '../generated/types/externalPluginAdapter.js';
import { getPluginEncoder } from '../generated/types/plugin.js';
import type { PluginAccountData } from './pluginAccountData.js';

/** Encodes complete accounts, recalculating registry offsets from the payloads. */
export function getPluginAccountDataEncoder<T>(
  baseEncoder: Encoder<T>
): Encoder<T & Partial<PluginAccountData>> {
  return transformEncoder(
    getBytesEncoder(),
    (value: T & Partial<PluginAccountData>) => {
      const base = baseEncoder.encode(value);
      const plugins = value.plugins ?? [];
      const externalPlugins = value.externalPluginAdapters ?? [];
      if (!value.pluginHeader && !plugins.length && !externalPlugins.length)
        return base;

      const payloads: ReadonlyUint8Array[] = [];
      let offset = BigInt(base.length + getPluginHeaderV1Size());
      const registry = plugins.map(({ plugin, ...record }) => {
        const bytes = getPluginEncoder().encode(plugin);
        const normalized = { ...record, offset };
        payloads.push(bytes);
        offset += BigInt(bytes.length);
        return normalized;
      });
      const externalRegistry = externalPlugins.map(
        ({ adapter, data, rawData, ...record }) => {
          const bytes = getExternalPluginAdapterEncoder().encode(adapter);
          const normalized = {
            ...record,
            dataLen: none<bigint>(),
            dataOffset: none<bigint>(),
            offset,
          };
          payloads.push(bytes);
          offset += BigInt(bytes.length);
          if (data !== undefined && rawData === undefined) {
            throw new Error(
              'External account encoding requires rawData for its data payload.'
            );
          }
          if (rawData !== undefined) {
            normalized.dataOffset = some(offset);
            normalized.dataLen = some(BigInt(rawData.length));
            payloads.push(rawData);
            offset += BigInt(rawData.length);
          }
          return normalized;
        }
      );
      const header = getPluginHeaderV1Encoder().encode({
        pluginRegistryOffset: offset,
      });
      const records = getPluginRegistryV1Encoder().encode({
        externalRegistry,
        registry,
      });
      const chunks = [base, header, ...payloads, records];
      const result = new Uint8Array(
        chunks.reduce((size, chunk) => size + chunk.length, 0)
      );
      let position = 0;
      for (const chunk of chunks) {
        result.set(chunk, position);
        position += chunk.length;
      }
      return result;
    }
  );
}
