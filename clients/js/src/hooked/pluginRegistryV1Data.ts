import {
  Serializer,
  array,
  option,
  tuple,
  u64,
} from '@metaplex-foundation/umi/serializers';
import {
  Key,
  PluginType,
  RegistryRecord,
  RegistryRecordArgs,
  getKeySerializer,
  getBasePluginAuthoritySerializer,
  getPluginTypeSerializer,
  ExternalPluginAdapterRegistryRecordArgs,
  ExternalPluginAdapterRegistryRecord,
  getExternalPluginAdapterTypeSerializer,
  ExternalPluginAdapterType,
  getHookableLifecycleEventSerializer,
  getExternalPluginAdapterCheckResultSerializer,
} from '../generated';
import {
  PluginRegistryV1AccountData,
  PluginRegistryV1AccountDataArgs,
} from '../generated/types/pluginRegistryV1AccountData';

export { PluginRegistryV1AccountData } from '../generated/types/pluginRegistryV1AccountData';

export type RegistryRecordWithUnknown = RegistryRecord & {
  isUnknown?: boolean;
};

export type ExternalPluginAdapterRegistryRecordWithUnknown =
  ExternalPluginAdapterRegistryRecord & {
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
        getBasePluginAuthoritySerializer().deserialize(
          buffer,
          pluginTypeOffset
        );
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

export function getAdapterRegistryRecordSerializer(): Serializer<
  ExternalPluginAdapterRegistryRecordArgs,
  ExternalPluginAdapterRegistryRecordWithUnknown
> {
  return {
    description: 'AdapterRegistryRecordWithUnknown',
    fixedSize: null,
    maxSize: null,
    serialize: () => {
      throw new Error('Operation not supported.');
    },
    deserialize: (
      buffer: Uint8Array,
      offset = 0
    ): [ExternalPluginAdapterRegistryRecordWithUnknown, number] => {
      let [pluginType, pluginTypeOffset, isUnknown] = [
        ExternalPluginAdapterType.DataStore,
        offset + 1,
        true,
      ];
      try {
        [pluginType, pluginTypeOffset] =
          getExternalPluginAdapterTypeSerializer().deserialize(buffer, offset);
        isUnknown = false;
      } catch (e) {
        // do nothing
      }

      const [authority, authorityOffset] =
        getBasePluginAuthoritySerializer().deserialize(
          buffer,
          pluginTypeOffset
        );
      const [lifecycleChecks, lifecycleChecksOffset] = option(
        array(
          tuple([
            getHookableLifecycleEventSerializer(),
            getExternalPluginAdapterCheckResultSerializer(),
          ])
        )
      ).deserialize(buffer, authorityOffset);

      const [pluginOffset, pluginOffsetOffset] = u64().deserialize(
        buffer,
        lifecycleChecksOffset
      );

      const [dataOffset, dataOffsetOffset] = option(u64()).deserialize(
        buffer,
        pluginOffsetOffset
      );
      const [dataLen, dataLenOffset] = option(u64()).deserialize(
        buffer,
        dataOffsetOffset
      );
      return [
        {
          pluginType,
          authority,
          lifecycleChecks,
          offset: pluginOffset,
          isUnknown,
          dataOffset,
          dataLen,
        },
        dataLenOffset,
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

      const [
        externalPluginAdapterRegistry,
        externalPluginAdapterRegistryOffset,
      ] = array(getAdapterRegistryRecordSerializer()).deserialize(
        buffer,
        registryOffset
      );

      return [
        {
          key,
          registry: registry.filter(
            (record: RegistryRecordWithUnknown) => !record.isUnknown
          ),
          externalPluginAdapterRegistry: externalPluginAdapterRegistry.filter(
            (record: ExternalPluginAdapterRegistryRecordWithUnknown) =>
              !record.isUnknown
          ),
        },
        externalPluginAdapterRegistryOffset,
      ];
    },
  };
}
