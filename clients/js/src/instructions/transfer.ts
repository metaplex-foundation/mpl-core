import { Context, publicKey } from '@metaplex-foundation/umi';
import { CollectionV1, transferV1, AssetV1 } from '../generated';
import { findExtraAccounts } from '../plugins';
import { deriveExternalPluginAdapters } from '../helpers';

export type TransferArgs = Omit<
  Parameters<typeof transferV1>[1],
  'asset' | 'collection'
> & {
  asset: Pick<AssetV1, 'publicKey' | 'owner' | 'oracles' | 'lifecycleHooks'>;
  collection?: Pick<CollectionV1, 'publicKey' | 'oracles' | 'lifecycleHooks'>;
};

export const transfer = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa'>,
  { asset, collection, ...args }: TransferArgs
) => {
  const derivedExternalPluginAdapters = deriveExternalPluginAdapters(
    asset,
    collection
  );

  const extraAccounts = findExtraAccounts(
    context,
    'transfer',
    derivedExternalPluginAdapters,
    {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      owner: asset.owner,
      recipient: publicKey(args.newOwner),
    }
  );

  return transferV1(context, {
    ...args,
    asset: asset.publicKey,
    collection: collection?.publicKey,
  }).addRemainingAccounts(extraAccounts);
};
