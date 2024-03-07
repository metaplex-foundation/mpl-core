import { Authority, PluginHeader, PluginRegistry, Plugin } from 'src/generated';

export * from './fetchAssetWithPlugins';
export * from './fetchCollectionWithPlugins';

export type PluginWithAuthority = {
  plugin: Plugin;
  authority: Authority;
};

export type PluginList = {
  pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  plugins?: PluginWithAuthority[];
  pluginRegistry?: Omit<PluginRegistry, 'publicKey' | 'header'>;
};
