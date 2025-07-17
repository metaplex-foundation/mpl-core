import { Serializer } from '@metaplex-foundation/umi/serializers';

import {
  Key,
  PluginHeaderV1,
  PluginHeaderV1AccountData,
  getPluginHeaderV1AccountDataSerializer,
} from '../generated';

import {
  GroupV1AccountData as GenGroupV1AccountData,
  GroupV1AccountDataArgs as GenGroupV1AccountDataArgs,
  getGroupV1AccountDataSerializer as genGetGroupV1AccountDataSerializer,
} from '../generated/accounts/groupV1';

import {
  ExternalPluginAdaptersList,
  GroupPluginsList,
  registryRecordsToPluginsList,
} from '../plugins';
import { externalRegistryRecordsToExternalPluginAdapterList } from '../plugins/externalPluginAdapters';
import {
  PluginRegistryV1AccountData,
  getPluginRegistryV1AccountDataSerializer,
} from './pluginRegistryV1Data';

export type GroupV1AccountData = GenGroupV1AccountData &
  GroupPluginsList &
  ExternalPluginAdaptersList & {
    pluginHeader?: Omit<PluginHeaderV1, 'publicKey' | 'header'>;
  };

export type GroupV1AccountDataArgs = GenGroupV1AccountDataArgs & {
  pluginHeader?: Omit<PluginHeaderV1, 'publicKey' | 'header'>;
};

export const getGroupV1AccountDataSerializer = (): Serializer<
  GroupV1AccountDataArgs,
  GroupV1AccountData
> => ({
  description: 'GroupAccountData',
  fixedSize: null,
  maxSize: null,
  serialize: () => {
    throw new Error('Operation not supported.');
  },
  deserialize: (
    buffer: Uint8Array,
    offset = 0
  ): [GroupV1AccountData, number] => {
    // Deserialize base group data
    const [group, groupOffset] =
      genGetGroupV1AccountDataSerializer().deserialize(buffer, offset);

    if (group.key !== Key.GroupV1) {
      throw new Error(`Expected a Group account, got key: ${group.key}`);
    }

    let pluginHeader: PluginHeaderV1AccountData | undefined;
    let pluginRegistry: PluginRegistryV1AccountData | undefined;
    let pluginsList: GroupPluginsList | undefined;
    let externalPluginAdaptersList: ExternalPluginAdaptersList | undefined;
    let finalOffset = groupOffset;

    if (buffer.length !== groupOffset) {
      [pluginHeader] = getPluginHeaderV1AccountDataSerializer().deserialize(
        buffer,
        groupOffset
      );

      [pluginRegistry, finalOffset] =
        getPluginRegistryV1AccountDataSerializer().deserialize(
          buffer,
          Number(pluginHeader.pluginRegistryOffset)
        );

      pluginsList = registryRecordsToPluginsList(
        pluginRegistry.registry,
        buffer
      );

      externalPluginAdaptersList =
        externalRegistryRecordsToExternalPluginAdapterList(
          pluginRegistry.externalRegistry,
          buffer
        );
    }

    return [
      {
        pluginHeader,
        ...pluginsList,
        ...externalPluginAdaptersList,
        ...group,
      },
      finalOffset,
    ];
  },
});
