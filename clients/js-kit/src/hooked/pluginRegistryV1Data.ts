import {
  createDecoder,
  getArrayDecoder,
  getU64Decoder,
  getOptionDecoder,
  getTupleDecoder,
  type Decoder,
} from '@solana/codecs';
import {
  Key,
  PluginType,
  RegistryRecord,
  ExternalRegistryRecord,
  getKeyDecoder,
  getBasePluginAuthorityDecoder,
  getPluginTypeDecoder,
  ExternalPluginAdapterType,
  getExternalPluginAdapterTypeDecoder,
  getHookableLifecycleEventDecoder,
  getExternalCheckResultDecoder,
} from '../generated';

export type PluginRegistryV1AccountData = {
  key: Key;
  registry: RegistryRecord[];
  externalRegistry: ExternalRegistryRecord[];
};

export type RegistryRecordWithUnknown = RegistryRecord & {
  isUnknown?: boolean;
};

export type ExternalRegistryRecordWithUnknown = ExternalRegistryRecord & {
  isUnknown?: boolean;
};

export function getRegistryRecordWithUnknownDecoder(): Decoder<RegistryRecordWithUnknown> {
  return createDecoder({
    read: (bytes, offset) => {
      let pluginType = PluginType.Attributes;
      let pluginTypeOffset = offset + 1;
      let isUnknown = true;

      try {
        [pluginType, pluginTypeOffset] = getPluginTypeDecoder().read(bytes, offset);
        isUnknown = false;
      } catch {
        // Unknown plugin type
      }

      const [authority, authorityOffset] = getBasePluginAuthorityDecoder().read(
        bytes,
        pluginTypeOffset
      );
      const [pluginOffset, pluginOffsetOffset] = getU64Decoder().read(
        bytes,
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
  });
}

export function getExternalRegistryRecordWithUnknownDecoder(): Decoder<ExternalRegistryRecordWithUnknown> {
  return createDecoder({
    read: (bytes, offset) => {
      let pluginType = ExternalPluginAdapterType.AppData;
      let pluginTypeOffset = offset + 1;
      let isUnknown = true;

      try {
        [pluginType, pluginTypeOffset] = getExternalPluginAdapterTypeDecoder().read(
          bytes,
          offset
        );
        isUnknown = false;
      } catch {
        // Unknown plugin type
      }

      const [authority, authorityOffset] = getBasePluginAuthorityDecoder().read(
        bytes,
        pluginTypeOffset
      );

      const lifecycleChecksDecoder = getOptionDecoder(
        getArrayDecoder(
          getTupleDecoder([
            getHookableLifecycleEventDecoder(),
            getExternalCheckResultDecoder(),
          ])
        )
      );
      const [lifecycleChecks, lifecycleChecksOffset] = lifecycleChecksDecoder.read(
        bytes,
        authorityOffset
      );

      const [pluginOffset, pluginOffsetOffset] = getU64Decoder().read(
        bytes,
        lifecycleChecksOffset
      );

      const optionU64Decoder = getOptionDecoder(getU64Decoder());
      const [dataOffset, dataOffsetOffset] = optionU64Decoder.read(
        bytes,
        pluginOffsetOffset
      );
      const [dataLen, dataLenOffset] = optionU64Decoder.read(bytes, dataOffsetOffset);

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
  });
}

export function getPluginRegistryV1AccountDataDecoder(): Decoder<PluginRegistryV1AccountData> {
  return createDecoder({
    read: (bytes, offset) => {
      const [key, keyOffset] = getKeyDecoder().read(bytes, offset);
      if (key !== Key.PluginRegistryV1) {
        throw new Error(`Expected a PluginRegistryV1 account, got key: ${key}`);
      }

      const registryDecoder = getArrayDecoder(getRegistryRecordWithUnknownDecoder());
      const [registry, registryOffset] = registryDecoder.read(bytes, keyOffset);

      const externalRegistryDecoder = getArrayDecoder(getExternalRegistryRecordWithUnknownDecoder());
      const [externalRegistry, externalRegistryOffset] = externalRegistryDecoder.read(
        bytes,
        registryOffset
      );

      return [
        {
          key,
          registry: registry.filter((record) => !record.isUnknown),
          externalRegistry: externalRegistry.filter((record) => !record.isUnknown),
        },
        externalRegistryOffset,
      ];
    },
  });
}
