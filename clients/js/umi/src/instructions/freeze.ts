import {
  Context,
  publicKey,
  PublicKey,
  Signer,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  AssetV1,
  CollectionV1,
  PluginType,
  addPluginV1,
  removePluginV1,
  revokePluginAuthorityV1,
  updatePluginV1,
} from '../generated';
import { isFrozen } from '../helpers';
import { createPlugin } from '../plugins';
import { addressPluginAuthority } from '../authority';

export type FreezeAssetArgs = {
  asset: AssetV1;
  delegate: PublicKey | Signer;
  authority?: Signer;
  collection?: CollectionV1;
};

/**
 * Freezes an asset and assigns the freeze delegate to the given address
 * @param context Umi context
 * @param args FreezeAssetArgs
 * @returns TransactionBuilder
 */
export function freezeAsset(
  context: Pick<Context, 'payer' | 'programs'>,
  { asset, authority, collection, delegate }: FreezeAssetArgs
) {
  if (isFrozen(asset, collection)) {
    throw new Error('Cannot freeze: asset is already frozen');
  }

  if (asset.freezeDelegate && asset.freezeDelegate.authority.type === 'None') {
    // not sure why this anyone would do this but we check it anyways
    throw new Error(
      'Cannot freeze: owner has made the freeze immutable until transfer'
    );
  }
  let txBuilder = transactionBuilder();

  if (asset.freezeDelegate) {
    if (asset.freezeDelegate.authority.type !== 'Owner') {
      txBuilder = txBuilder.add(
        revokePluginAuthorityV1(context, {
          asset: asset.publicKey,
          collection: collection?.publicKey,
          pluginType: PluginType.FreezeDelegate,
          authority,
        })
      );
    }
    txBuilder = txBuilder.add(
      removePluginV1(context, {
        asset: asset.publicKey,
        collection: collection?.publicKey,
        pluginType: PluginType.FreezeDelegate,
        authority,
      })
    );
  }

  txBuilder = txBuilder.add(
    addPluginV1(context, {
      asset: asset.publicKey,
      collection: collection?.publicKey,
      plugin: createPlugin({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
      initAuthority: addressPluginAuthority(publicKey(delegate)),
      authority,
    })
  );

  return txBuilder;
}

export type ThawAssetArgs = {
  asset: AssetV1;
  delegate: Signer;
  collection?: CollectionV1;
};

/**
 * Thaws an asset and assigns the freeze delegate back to the owner
 * @param context Umi context
 * @param args ThawAssetArgs
 * @returns TransactionBuilder
 */
export function thawAsset(
  context: Pick<Context, 'payer' | 'programs'>,
  { asset, delegate, collection }: ThawAssetArgs
) {
  if (!isFrozen(asset, collection)) {
    throw new Error('Cannot thaw: asset is not frozen');
  }

  return transactionBuilder()
    .add(
      updatePluginV1(context, {
        asset: asset.publicKey,
        collection: collection?.publicKey,
        plugin: createPlugin({
          type: 'FreezeDelegate',
          data: { frozen: false },
        }),
        authority: delegate,
      })
    )
    .add(
      revokePluginAuthorityV1(context, {
        asset: asset.publicKey,
        collection: collection?.publicKey,
        pluginType: PluginType.FreezeDelegate,
        authority: delegate,
      })
    );
}
