import { Authority, PluginHeader, PluginRegistry, Plugin } from 'src/generated';

export * from './fetchAssetWithPlugins';
export * from './fetchCollectionWithPlugins';

export type PluginWithAuthorities = {
    plugin: Plugin;
    authorities: Authority[];
};

export type PluginList = {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
    plugins?: PluginWithAuthorities[];
    pluginRegistry?: Omit<PluginRegistry, 'publicKey' | 'header'>;
};