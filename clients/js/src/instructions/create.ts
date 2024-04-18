import { Context, publicKey } from '@metaplex-foundation/umi';
import { CollectionV1, createV2, ExternalPluginSchema } from '../generated';
import {
  PluginAuthorityPairHelperArgsV2,
  createExternalPluginInitInfo,
  findExtraAccounts,
  pluginAuthorityPairV2,
} from '../plugins';
import { deriveExternalPlugins } from '../helpers';
import {
  ExternalPluginInitInfoArgs,
  ExternalPluginsList,
} from '../plugins/externalPlugins';

export type CreateArgs = Omit<
  Parameters<typeof createV2>[1],
  'plugins' | 'externalPlugins' | 'collection'
> & {
  collection?: CollectionV1;
  plugins?: PluginAuthorityPairHelperArgsV2[];
  externalPlugins?: ExternalPluginInitInfoArgs[];
};

export const create = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, plugins, externalPlugins, collection, ...args }: CreateArgs
) => {
  const owner = args.owner || args.updateAuthority || args.payer;

  const assetExternalPlugins: ExternalPluginsList = {
    oracles: [],
    lifecycleHooks: [],
  };

  externalPlugins?.forEach((plugin) => {
    switch (plugin.type) {
      case 'Oracle':
        assetExternalPlugins.oracles?.push({
          ...plugin,
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
      default:
        assetExternalPlugins.lifecycleHooks?.push({
          ...plugin,
          hookedProgram: plugin.hookedProgram,
          authority: plugin.initPluginAuthority || {
            type: 'UpdateAuthority',
          },
          dataLen: 0,
          dataOffset: 0,
          type: 'LifecycleHook',
          schema: plugin.schema || ExternalPluginSchema.Binary,
        });
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
    plugins: plugins?.map(pluginAuthorityPairV2) || [],
    externalPlugins: externalPlugins?.map(createExternalPluginInitInfo) || [],
    asset,
    collection: collection ? collection.publicKey : undefined,
  }).addRemainingAccounts(extraAccounts);
};
