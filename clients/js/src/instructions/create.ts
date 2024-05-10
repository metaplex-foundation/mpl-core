import { Context, publicKey } from '@metaplex-foundation/umi';
import { CollectionV1, createV2, PluginAdapterSchema } from '../generated';
import {
  createPluginAdapterInitInfo,
  findExtraAccounts,
  PluginAuthorityPairArgsV2,
  pluginAuthorityPairV2,
} from '../plugins';
import { derivePluginAdapters } from '../helpers';
import {
  PluginAdapterInitInfoArgs,
  PluginAdaptersList,
  isPluginAdapterType,
} from '../plugins/pluginAdapters';

export type CreateArgsPlugin =
  | PluginAuthorityPairArgsV2
  | PluginAdapterInitInfoArgs;

export type CreateArgs = Omit<
  Parameters<typeof createV2>[1],
  'plugins' | 'pluginAdapters' | 'collection'
> & {
  collection?: Pick<CollectionV1, 'publicKey' | 'oracles' | 'lifecycleHooks'>;
  plugins?: CreateArgsPlugin[];
};

export const create = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, plugins, collection, ...args }: CreateArgs
) => {
  const owner = args.owner || args.updateAuthority || args.payer;

  const assetPluginAdapters: PluginAdaptersList = {
    oracles: [],
    lifecycleHooks: [],
  };

  const pluginAdapters: PluginAdapterInitInfoArgs[] = [];
  const firstPartyPlugins: PluginAuthorityPairArgsV2[] = [];

  // Create dummy plugin adapters to resuse findExtraAccounts method
  plugins?.forEach((plugin) => {
    if (isPluginAdapterType(plugin)) {
      pluginAdapters.push(plugin as PluginAdapterInitInfoArgs);
      switch (plugin.type) {
        case 'Oracle':
          assetPluginAdapters.oracles?.push({
            ...plugin,
            resultsOffset: plugin.resultsOffset || { type: 'NoOffset' },
            baseAddress: plugin.baseAddress,
            authority: plugin.initPluginAuthority || {
              type: 'UpdateAuthority',
            },
            type: 'Oracle',
          });
          break;
        case 'DataStore':
          // Do nothing, datastore has no extra accounts
          break;
        case 'LifecycleHook':
          assetPluginAdapters.lifecycleHooks?.push({
            ...plugin,
            hookedProgram: plugin.hookedProgram,
            authority: plugin.initPluginAuthority || {
              type: 'UpdateAuthority',
            },
            type: 'LifecycleHook',
            schema: plugin.schema || PluginAdapterSchema.Binary,
          });
          break;
        default:
        // Do nothing
      }
    } else {
      firstPartyPlugins.push(plugin as PluginAuthorityPairArgsV2);
    }
  });

  const derivedPluginAdapters = derivePluginAdapters(
    assetPluginAdapters,
    collection
  );
  const extraAccounts = findExtraAccounts(
    context,
    'create',
    derivedPluginAdapters,
    {
      asset: asset.publicKey,
      collection: collection ? collection.publicKey : undefined,
      // need to replicate program behavior
      owner: owner ? publicKey(owner) : context.identity.publicKey,
    }
  );

  return createV2(context, {
    ...args,
    plugins: firstPartyPlugins.map(pluginAuthorityPairV2),
    pluginAdapters: pluginAdapters.map(createPluginAdapterInitInfo),
    asset,
    collection: collection ? collection.publicKey : undefined,
  }).addRemainingAccounts(extraAccounts);
};
