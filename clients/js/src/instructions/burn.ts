import { Context } from '@metaplex-foundation/umi';
import { CollectionV1, burnV1, AssetV1 } from '../generated';
import { findExtraAccounts } from '../plugins';
import { deriveExternalPlugins } from '../helpers';

export type BurnArgs = Omit<
  Parameters<typeof burnV1>[1],
  'asset' | 'collection'
> & {
  asset: AssetV1;
  collection?: CollectionV1;
};

export const burn = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, collection, ...args }: BurnArgs
) => {
  const derivedExternalPlugins = deriveExternalPlugins(asset, collection);

  const extraAccounts = findExtraAccounts(
    context,
    'burn',
    derivedExternalPlugins,
    {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      owner: asset.owner,
    }
  );

  return burnV1(context, {
    ...args,
    asset: asset.publicKey,
    collection: collection?.publicKey,
  }).addRemainingAccounts(extraAccounts);
};
