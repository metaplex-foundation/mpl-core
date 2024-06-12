import { Context } from '@metaplex-foundation/umi';
import {
  CollectionV1,
  AssetV1,
  UpdateV2InstructionDataArgs,
  updateV2,
} from '../generated';
import { findExtraAccounts } from '../plugins';
import { deriveExternalPluginAdapters } from '../helpers';

export type UpdateArgs = Omit<
  Parameters<typeof updateV2>[1],
  'asset' | 'collection' | 'newName' | 'newUri'
> & {
  asset: Pick<AssetV1, 'publicKey' | 'owner' | 'oracles' | 'lifecycleHooks'>;
  collection?: Pick<CollectionV1, 'publicKey' | 'oracles' | 'lifecycleHooks'>;
  name?: UpdateV2InstructionDataArgs['newName'];
  uri?: UpdateV2InstructionDataArgs['newUri'];
};

export const update = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa'>,
  { asset, collection, name, uri, ...args }: UpdateArgs
) => {
  const derivedExternalPluginAdapters = deriveExternalPluginAdapters(
    asset,
    collection
  );

  const extraAccounts = findExtraAccounts(
    context,
    'update',
    derivedExternalPluginAdapters,
    {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      owner: asset.owner,
    }
  );

  return updateV2(context, {
    ...args,
    asset: asset.publicKey,
    collection: collection?.publicKey,
    newName: name,
    newUri: uri,
  }).addRemainingAccounts(extraAccounts);
};
