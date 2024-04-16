import { Context } from "@metaplex-foundation/umi";
import { CollectionV1, updateV1, AssetV1 } from "../generated";
import { findExtraAccounts } from "../plugins";
import { deriveExternalPlugins } from "../helpers";

export type UpdateArgs = Omit<Parameters<typeof updateV1>[1], 'asset' | 'collection'> & {
  asset: AssetV1,
  collection?: CollectionV1,
}

export const update = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>, {
    asset,
    collection,
    ...args
  }: UpdateArgs) => {
    const derivedExternalPlugins = deriveExternalPlugins(asset, collection)

    const extraAccounts = findExtraAccounts(context, 'update', derivedExternalPlugins, {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      owner: asset.owner,
    })

    return updateV1(context, {
      ...args,
      asset: asset.publicKey,
      collection: collection?.publicKey,
    }).addRemainingAccounts(extraAccounts)
  }