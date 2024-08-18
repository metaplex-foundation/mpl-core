import test from 'ava';

import { generateSigner, lamports } from '@metaplex-foundation/umi';
import { createAssetWithCollection } from '../../_setupSdk';
import {
  createCollectionV2,
  pluginAuthorityPairV2,
  addPlugin,
  createV2,
  addCollectionPlugin,
  transfer,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can add sol transfer fee to asset', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {});

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: {
      type: 'SolTransferFee',
      feeAmount: lamports(1_000_000),
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    solTransferFee: {
      authority: {
        type: 'UpdateAuthority',
      },
      feeAmount: lamports(1_000_000),
    },
  });
});

test('it can create asset with sol transfer fee', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(umi, {
    plugins: [
      {
        type: 'SolTransferFee',
        feeAmount: lamports(1_000_000),
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    solTransferFee: {
      authority: {
        type: 'UpdateAuthority',
      },
      feeAmount: lamports(1_000_000),
    },
  });
});

test('it cannot create sol transfer fee without a collection', async (t) => {
  const umi = await createUmi();
  const asset = generateSigner(umi);

  const result = createV2(umi, {
    asset,
    plugins: [
      pluginAuthorityPairV2({
        type: 'SolTransferFee',
        feeAmount: lamports(1_000_000),
      }),
    ],
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'PluginRequiresCollection',
  });
});

test('it cannot add sol transfer fee to a collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'SolTransferFee',
      feeAmount: lamports(1_000_000),
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'PluginNotAllowedOnCollection',
  });
});

test('it cannot create collection with sol transfer fee', async (t) => {
  const umi = await createUmi();
  const collection = generateSigner(umi);

  const result = createCollectionV2(umi, {
    ...DEFAULT_COLLECTION,
    collection,
    plugins: [
      pluginAuthorityPairV2({
        type: 'SolTransferFee',
        feeAmount: lamports(1_000_000),
      }),
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'PluginNotAllowedOnCollection',
  });
});

test('it pays SOL on transfer', async (t) => {
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {
    plugins: [
      {
        type: 'SolTransferFee',
        feeAmount: lamports(1_000_000),
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    solTransferFee: {
      authority: {
        type: 'UpdateAuthority',
      },
      feeAmount: lamports(1_000_000),
    },
  });

  const identityBeforeBalance = await umi.rpc.getBalance(
    umi.identity.publicKey
  );
  const collectionBeforeBalance = await umi.rpc.getBalance(
    collection.publicKey
  );

  await transfer(umi, {
    asset,
    collection,
    newOwner: recipient.publicKey,
  }).sendAndConfirm(umi);

  const identityAfterBalance = await umi.rpc.getBalance(umi.identity.publicKey);
  const collectionAfterBalance = await umi.rpc.getBalance(collection.publicKey);

  const identityExpected =
    identityBeforeBalance.basisPoints -
    1_000_000n - // Transfer fee
    5_000n; // Transaction fee
  t.is(identityExpected, identityAfterBalance.basisPoints);
  t.is(
    collectionBeforeBalance.basisPoints + 1_000_000n,
    collectionAfterBalance.basisPoints
  );
});
