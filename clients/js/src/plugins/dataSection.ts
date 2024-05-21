import {
  BaseDataSection,
  BaseDataSectionInitInfoArgs,
  BaseDataSectionUpdateInfoArgs,
  ExternalPluginAdapterSchema,
  ExternalRegistryRecord,
  LinkedDataKey,
} from '../generated';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { parseExternalPluginAdapterData } from './lib';
import {
  PluginAuthority,
  pluginAuthorityFromBase,
} from './pluginAuthority';

export type DataSection = Omit<BaseDataSection, 'dataAuthority'> & {
  dataAuthority: PluginAuthority;
  data?: any;
};

export type DataSectionPlugin = BaseExternalPluginAdapter &
  DataSection & {
    type: 'DataSection';
    managerKey: LinkedDataKey;
    data: any;
  };

export type DataSectionInitInfoArgs = BaseDataSectionInitInfoArgs & {
  type: 'DataSection';
  managerKey: LinkedDataKey;
};

export type DataSectionUpdateInfoArgs =
  BaseDataSectionUpdateInfoArgs & {
    key: ExternalPluginAdapterKey;
  };

export function dataSectionInitInfoArgsToBase(
  d: DataSectionInitInfoArgs
): BaseDataSectionInitInfoArgs {
  return {
    managerKey: d.managerKey,
  };
}

export function dataSectionUpdateInfoArgsToBase(
  d: DataSectionUpdateInfoArgs
): BaseDataSectionUpdateInfoArgs {
  return {};
}

export function dataSectionFromBase(
  s: BaseDataSection,
  r: ExternalRegistryRecord,
  account: Uint8Array
): DataSection {
  return {
    ...s,
    data: parseExternalPluginAdapterData(s, r, account),
  };
}

export const dataSectionManifest: ExternalPluginAdapterManifest<
  DataSection,
  BaseDataSection,
  DataSectionInitInfoArgs,
  BaseDataSectionInitInfoArgs,
  DataSectionUpdateInfoArgs,
  BaseDataSectionUpdateInfoArgs
> = {
  type: 'DataSection',
  fromBase: dataSectionFromBase,
  initToBase: dataSectionInitInfoArgsToBase,
  updateToBase: dataSectionUpdateInfoArgsToBase,
};
