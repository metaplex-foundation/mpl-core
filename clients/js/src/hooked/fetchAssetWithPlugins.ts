import { Context, Pda, PublicKey, RpcGetAccountOptions, assertAccountExists, publicKey as toPublicKey } from "@metaplex-foundation/umi";
import { Asset, Authority, Plugin, PluginHeader, PluginRegistry, deserializeAsset, getAssetAccountDataSerializer, getPluginHeaderAccountDataSerializer, getPluginRegistryAccountDataSerializer, getPluginSerializer } from "../generated";

export type PluginWithAuthorities = { plugin: Plugin, authorities: Authority[] };

export type PluginList = {
    pluginHeader: Omit<PluginHeader, 'publicKey' | 'header'>,
    plugins: PluginWithAuthorities[],
    pluginRegistry: Omit<PluginRegistry, 'publicKey' | 'header'>,
};
export type AssetWithPlugins = Asset & PluginList;

export async function fetchAssetWithPlugins(
    context: Pick<Context, 'rpc'>,
    publicKey: PublicKey | Pda,
    options?: RpcGetAccountOptions
): Promise<AssetWithPlugins> {
    const maybeAccount = await context.rpc.getAccount(
        toPublicKey(publicKey, false),
        options
    );
    assertAccountExists(maybeAccount, 'Asset');
    const asset = deserializeAsset(maybeAccount);
    const assetData = getAssetAccountDataSerializer().serialize(asset);
    const pluginHeader = getPluginHeaderAccountDataSerializer().deserialize(maybeAccount.data, assetData.length)[0];
    const pluginRegistry = getPluginRegistryAccountDataSerializer().deserialize(maybeAccount.data, Number(pluginHeader.pluginRegistryOffset))[0];
    const plugins = pluginRegistry.registry.map((record) => ({
        plugin: getPluginSerializer().deserialize(maybeAccount.data, Number(record.data.offset))[0],
        authorities: record.data.authorities,
    }));

    const assetWithPlugins: AssetWithPlugins = {
        pluginHeader,
        plugins,
        pluginRegistry,
        ...asset
    };

    return assetWithPlugins;
}