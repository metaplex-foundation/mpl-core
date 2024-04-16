import { Serializer, array, u64 } from '@metaplex-foundation/umi/serializers';
import { some } from '@metaplex-foundation/umi';
import {
  Key,
  PluginType,
  RegistryRecord,
  RegistryRecordArgs,
  getKeySerializer,
  getBasePluginAuthoritySerializer,
  getPluginTypeSerializer,
  ExternalRegistryRecordArgs,
  ExternalRegistryRecord,
} from '../generated';
import {
  PluginRegistryV1AccountData,
  PluginRegistryV1AccountDataArgs,
} from '../generated/types/pluginRegistryV1AccountData';
import { nonePluginAuthority } from '../authority';

export { PluginRegistryV1AccountData } from '../generated/types/pluginRegistryV1AccountData';

export type RegistryRecordWithUnknown = RegistryRecord & {
  isUnknown?: boolean;
};

export type ExternalRegistryRecordWithUnknown = ExternalRegistryRecord & {
  isUnknown?: boolean;
};

export function getRegistryRecordSerializer(): Serializer<
  RegistryRecordArgs,
  RegistryRecordWithUnknown
> {
  return {
    description: 'RegistryRecordWithUnknown',
    fixedSize: null,
    maxSize: null,
    serialize: () => {
      throw new Error('Operation not supported.');
    },
    deserialize: (
      buffer: Uint8Array,
      offset = 0
    ): [RegistryRecordWithUnknown, number] => {
      let [pluginType, pluginTypeOffset, isUnknown] = [
        PluginType.Attributes,
        offset + 1,
        true,
      ];
      try {
        [pluginType, pluginTypeOffset] = getPluginTypeSerializer().deserialize(
          buffer,
          offset
        );
        isUnknown = false;
      } catch (e) {
        // Do nothing, unknown plugin type
      }
      const [authority, authorityOffset] =
        getBasePluginAuthoritySerializer().deserialize(buffer, pluginTypeOffset);
      const [pluginOffset, pluginOffsetOffset] = u64().deserialize(
        buffer,
        authorityOffset
      );

      return [
        {
          pluginType,
          authority,
          offset: pluginOffset,
          isUnknown,
        },
        pluginOffsetOffset,
      ];
    },
  };
}

export function getExternalRegistryRecordSerializer(): Serializer<
  ExternalRegistryRecordArgs,
  ExternalRegistryRecordWithUnknown
> {
  return {
    description: 'ExternalRegistryRecordWithUnknown',
    fixedSize: null,
    maxSize: null,
    serialize: () => {
      throw new Error('Operation not supported.');
    },
    deserialize: (
      buffer: Uint8Array,
      offset = 0
    ): [ExternalRegistryRecordWithUnknown, number] => {

      try {
        const [externalRegistryRecord, externalRegistryRecordOffset] = getExternalRegistryRecordSerializer().deserialize(
          buffer,
          offset
        )
        return [{
          ...externalRegistryRecord,
          isUnknown: false,
        }, externalRegistryRecordOffset]
      } catch (e) {
        // TODO need to find the right offset for the next item
      }

      return [
        {
          pluginKey: {
            __kind: 'DataStore',
            fields: [nonePluginAuthority()],
          },
          authority: nonePluginAuthority(),
          lifecycleChecks: some([]),
          offset: BigInt(0),
        },
        offset,
      ];
    },
  };
}

export function getPluginRegistryV1AccountDataSerializer(): Serializer<
  PluginRegistryV1AccountDataArgs,
  PluginRegistryV1AccountData
> {
  return {
    description: 'PluginRegistryV1AccountData',
    fixedSize: null,
    maxSize: null,
    serialize: () => {
      throw new Error('Operation not supported.');
    },
    deserialize: (
      buffer: Uint8Array,
      offset = 0
    ): [PluginRegistryV1AccountData, number] => {
      const [key, keyOffset] = getKeySerializer().deserialize(buffer, offset);
      if (key !== Key.PluginRegistryV1) {
        throw new Error(`Expected a PluginRegistryV1 account, got key: ${key}`);
      }

      const [registry, registryOffset] = array(
        getRegistryRecordSerializer()
      ).deserialize(buffer, keyOffset);

      const [externalRegistry, externalRegistryOffset] = array(
        getExternalRegistryRecordSerializer()
      ).deserialize(buffer, registryOffset);

      return [
        {
          key,
          registry: registry.filter(
            (record: RegistryRecordWithUnknown) => !record.isUnknown
          ),
          externalRegistry: externalRegistry.filter(
            (record: ExternalRegistryRecordWithUnknown) => !record.isUnknown
          ),
        },
        externalRegistryOffset,
      ];
    },
  };
}
