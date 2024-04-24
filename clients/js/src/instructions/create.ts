import { Context, publicKey } from '@metaplex-foundation/umi';
import { CollectionV1, createV2, ExternalPluginSchema } from '../generated';
import {
  createExternalPluginInitInfo,
  findExtraAccounts,
  PluginArgsV2,
  pluginAuthorityPairV2,
} from '../plugins';
import { deriveExternalPlugins } from '../helpers';
import {
  ExternalPluginInitInfoArgs,
  ExternalPluginsList,
  isExternalPluginType,
} from '../plugins/externalPlugins';

export type CreateArgs = Omit<
  Parameters<typeof createV2>[1],
  'plugins' | 'externalPlugins' | 'collection'
> & {
  collection?: CollectionV1;
  plugins?: (PluginArgsV2 | ExternalPluginInitInfoArgs)[];
};

export const create = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, plugins, collection, ...args }: CreateArgs
) => {
  const owner = args.owner || args.updateAuthority || args.payer;

  const assetExternalPlugins: ExternalPluginsList = {
    oracles: [],
    lifecycleHooks: [],
  };

  const externalPlugins: ExternalPluginInitInfoArgs[] = [];
  const firstPartyPlugins: PluginArgsV2[] = [];

  // Create dummy external plugins to resuse findExtraAccounts method
  plugins?.forEach((plugin) => {
    if (isExternalPluginType(plugin)) {
      externalPlugins.push(plugin as ExternalPluginInitInfoArgs);
      switch (plugin.type) {
        case 'Oracle':
          assetExternalPlugins.oracles?.push({
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
          assetExternalPlugins.lifecycleHooks?.push({
            ...plugin,
            hookedProgram: plugin.hookedProgram,
            authority: plugin.initPluginAuthority || {
              type: 'UpdateAuthority',
            },
            dataLen: 0n,
            dataOffset: 0n,
            type: 'LifecycleHook',
            schema: plugin.schema || ExternalPluginSchema.Binary,
          });
          break;
        default:
        // Do nothing
      }
    } else {
      firstPartyPlugins.push(plugin as PluginArgsV2);
    }
  });

  const derivedExternalPlugins = deriveExternalPlugins(
    assetExternalPlugins,
    collection
  );
  const extraAccounts = findExtraAccounts(
    context,
    'create',
    derivedExternalPlugins,
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
    externalPlugins: externalPlugins.map(createExternalPluginInitInfo),
    asset,
    collection: collection ? collection.publicKey : undefined,
  }).addRemainingAccounts(extraAccounts);
};
