import { Context } from '@metaplex-foundation/umi';
import {
  CollectionV1,
  updateV1,
  AssetV1,
  UpdateV1InstructionDataArgs,
} from '../generated';
import { findExtraAccounts } from '../plugins';
import { derivePluginAdapters } from '../helpers';

export type UpdateArgs = Omit<
  Parameters<typeof updateV1>[1],
  'asset' | 'collection' | 'newName' | 'newUri'
> & {
  asset: Pick<AssetV1, 'publicKey' | 'owner' | 'oracles' | 'lifecycleHooks'>;
  collection?: Pick<CollectionV1, 'publicKey' | 'oracles' | 'lifecycleHooks'>;
  name?: UpdateV1InstructionDataArgs['newName'];
  uri?: UpdateV1InstructionDataArgs['newUri'];
};

export const update = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, collection, name, uri, ...args }: UpdateArgs
) => {
  const derivedPluginAdapters = derivePluginAdapters(asset, collection);

  const extraAccounts = findExtraAccounts(
    context,
    'update',
    derivedPluginAdapters,
    {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      owner: asset.owner,
    }
  );

  return updateV1(context, {
    ...args,
    asset: asset.publicKey,
    collection: collection?.publicKey,
    newName: name,
    newUri: uri,
  }).addRemainingAccounts(extraAccounts);
};
