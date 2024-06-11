import {
  BaseDataSection,
  BaseDataSectionInitInfoArgs,
  BaseDataSectionUpdateInfoArgs,
  ExternalRegistryRecord,
} from '../generated';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { parseExternalPluginAdapterData } from './lib';
import {
  LinkedDataKey,
  linkedDataKeyFromBase,
  linkedDataKeyToBase,
} from './linkedDataKey';
import { PluginAuthority, pluginAuthorityFromBase } from './pluginAuthority';

export type DataSection = Omit<
  BaseDataSection,
  'dataAuthority' | 'parentKey'
> & {
  dataAuthority?: PluginAuthority;
  parentKey: LinkedDataKey;
  data?: any;
};

export type DataSectionPlugin = BaseExternalPluginAdapter &
  DataSection & {
    type: 'DataSection';
  };

export type DataSectionInitInfoArgs = Omit<
  BaseDataSectionInitInfoArgs,
  'parentKey'
> & {
  type: 'DataSection';
  parentKey: LinkedDataKey;
};

export type DataSectionUpdateInfoArgs = BaseDataSectionUpdateInfoArgs & {
  key: ExternalPluginAdapterKey;
};

export function dataSectionInitInfoArgsToBase(
  d: DataSectionInitInfoArgs
): BaseDataSectionInitInfoArgs {
  return {
    parentKey: linkedDataKeyToBase(d.parentKey),
    schema: d.schema,
  };
}

export function dataSectionUpdateInfoArgsToBase(
  d: DataSectionUpdateInfoArgs
): BaseDataSectionUpdateInfoArgs {
  // You can't update the data section directly
  return {};
}

export function dataSectionFromBase(
  s: BaseDataSection,
  r: ExternalRegistryRecord,
  account: Uint8Array
): DataSection {
  return {
    ...s,
    parentKey: linkedDataKeyFromBase(s.parentKey),
    dataAuthority:
      s.parentKey.__kind !== 'LinkedLifecycleHook'
        ? pluginAuthorityFromBase(s.parentKey.fields[0])
        : undefined,
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
