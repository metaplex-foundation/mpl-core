import { Context, publicKey } from '@metaplex-foundation/umi';
import {
  CollectionV1,
  createV2,
  ExternalPluginAdapterSchema,
} from '../generated';
import {
  createExternalPluginAdapterInitInfo,
  findExtraAccounts,
  AssetPluginAuthorityPairArgsV2,
  pluginAuthorityPairV2,
} from '../plugins';
import { deriveExternalPluginAdapters } from '../helpers';
import {
  ExternalPluginAdapterInitInfoArgs,
  ExternalPluginAdaptersList,
  isExternalPluginAdapterType,
} from '../plugins/externalPluginAdapters';

export type CreateArgsPlugin =
  | AssetPluginAuthorityPairArgsV2
  | ExternalPluginAdapterInitInfoArgs;

export type CreateArgs = Omit<
  Parameters<typeof createV2>[1],
  'plugins' | 'externalPluginAdapters' | 'collection'
> & {
  collection?: Pick<CollectionV1, 'publicKey' | 'oracles' | 'lifecycleHooks'>;
  plugins?: CreateArgsPlugin[];
};

export const create = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, plugins, collection, ...args }: CreateArgs
) => {
  const owner = args.owner || args.updateAuthority || args.payer;

  const assetExternalPluginAdapters: ExternalPluginAdaptersList = {
    oracles: [],
    lifecycleHooks: [],
  };

  const externalPluginAdapters: ExternalPluginAdapterInitInfoArgs[] = [];
  const firstPartyPlugins: AssetPluginAuthorityPairArgsV2[] = [];

  // Create dummy external plugin adapters to resuse findExtraAccounts method
  plugins?.forEach((plugin) => {
    if (isExternalPluginAdapterType(plugin)) {
      externalPluginAdapters.push(plugin as ExternalPluginAdapterInitInfoArgs);
      switch (plugin.type) {
        case 'Oracle':
          assetExternalPluginAdapters.oracles?.push({
            ...plugin,
            resultsOffset: plugin.resultsOffset || { type: 'NoOffset' },
            baseAddress: plugin.baseAddress,
            authority: plugin.initPluginAuthority || {
              type: 'UpdateAuthority',
            },
            type: 'Oracle',
          });
          break;
        case 'AppData':
          // Do nothing, App Data has no extra accounts
          break;
        case 'LifecycleHook':
          assetExternalPluginAdapters.lifecycleHooks?.push({
            ...plugin,
            hookedProgram: plugin.hookedProgram,
            authority: plugin.initPluginAuthority || {
              type: 'UpdateAuthority',
            },
            type: 'LifecycleHook',
            schema: plugin.schema || ExternalPluginAdapterSchema.Binary,
          });
          break;
        default:
        // Do nothing
      }
    } else {
      firstPartyPlugins.push(plugin as AssetPluginAuthorityPairArgsV2);
    }
  });

  const derivedExternalPluginAdapters = deriveExternalPluginAdapters(
    assetExternalPluginAdapters,
    collection
  );
  const extraAccounts = findExtraAccounts(
    context,
    'create',
    derivedExternalPluginAdapters,
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
    externalPluginAdapters: externalPluginAdapters.map(
      createExternalPluginAdapterInitInfo
    ),
    asset,
    collection: collection ? collection.publicKey : undefined,
  }).addRemainingAccounts(extraAccounts);
};
