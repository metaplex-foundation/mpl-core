import { Authority, PluginHeader, PluginRegistry, Plugin } from 'src/generated';

export * from './fetchAssetWithPlugins';
export * from './fetchAssetWithPluginsTest';
export * from './fetchCollectionWithPlugins';
export * from './authorityHelpers';
export * from './pluginHelpers';
export * from './types';

export type PluginWithAuthority = {
  plugin: Plugin;
  authority: Authority;
};

export type PluginList = {
  pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  plugins?: PluginWithAuthority[];
  pluginRegistry?: Omit<PluginRegistry, 'publicKey' | 'header'>;
};
