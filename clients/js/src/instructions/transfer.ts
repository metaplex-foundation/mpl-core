import { Context, publicKey } from "@metaplex-foundation/umi";
import { CollectionV1, transferV1, AssetV1 } from "../generated";
import { findExtraAccounts } from "../plugins";
import { deriveExternalPlugins } from "../helpers";


export type TransferArgs = Omit<Parameters<typeof transferV1>[1], 'asset' | 'collection'> & {
  asset: AssetV1,
  collection?: CollectionV1,
}

export const transfer = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>, {
    asset,
    collection,
    ...args
  }: TransferArgs) => {
    const derivedExternalPlugins = deriveExternalPlugins(asset, collection)

    const extraAccounts = findExtraAccounts(context, 'transfer', derivedExternalPlugins, {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      owner: asset.owner,
      recipient: publicKey(args.newOwner),
    })

    return transferV1(context, {
      ...args,
      asset: asset.publicKey,
      collection: collection?.publicKey,
    }).addRemainingAccounts(extraAccounts)
  }