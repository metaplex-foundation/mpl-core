import test from 'ava';
import {
  generateSigner,
  assertAccountExists,
  sol,
} from '@metaplex-foundation/umi';
import {
  pluginAuthorityPair,
  burnV1,
  Key,
  transferV1,
  updatePluginAuthority,
  removePluginV1,
  PluginType,
  addPluginV1,
  createPlugin,
} from '../../../src';
import {
  assertAsset,
  assertBurned,
  createAsset,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it can burn an assets as an owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  const afterAsset = await assertBurned(t, umi, asset.publicKey);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
});

test('it can burn an assets as a delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    payer: owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await burnV1(umi, {
    payer: owner,
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  const afterAsset = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});

test('it can burn an assets as a delegate for a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const firstAssetOwner = generateSigner(umi);
  await umi.rpc.airdrop(firstAssetOwner.publicKey, sol(10));
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const collection = await createCollection(umi, {
    payer: firstAssetOwner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  const asset1 = await createAsset(umi, {
    authority: firstAssetOwner,
    owner: firstAssetOwner,
    collection: collection.publicKey,
  });

  const asset2 = await createAsset(umi, {
    authority: firstAssetOwner,
    owner: firstAssetOwner,
    collection: collection.publicKey,
  });

  // move asset #1 twice as a delegate for collection
  await transferV1(umi, {
    authority: firstAssetOwner,
    asset: asset1.publicKey,
    collection: collection.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: newOwner,
    asset: asset1.publicKey,
    collection: collection.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  // move asset #2 twice as a delegate for collection
  await transferV1(umi, {
    authority: firstAssetOwner,
    asset: asset2.publicKey,
    collection: collection.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: newOwner,
    asset: asset2.publicKey,
    collection: collection.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await burnV1(umi, {
    payer: firstAssetOwner,
    asset: asset1.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await burnV1(umi, {
    payer: firstAssetOwner,
    asset: asset2.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  const afterAsset1 = await umi.rpc.getAccount(asset1.publicKey);
  t.true(afterAsset1.exists);
  assertAccountExists(afterAsset1);
  t.deepEqual(afterAsset1.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset1.data.length, 1);
  t.is(afterAsset1.data[0], Key.Uninitialized);

  const afterAsset2 = await umi.rpc.getAccount(asset2.publicKey);
  t.true(afterAsset2.exists);
  assertAccountExists(afterAsset2);
  t.deepEqual(afterAsset2.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset2.data.length, 1);
  t.is(afterAsset2.data[0], Key.Uninitialized);
});

test('it can burn an asset which is the part of a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const firstAssetOwner = generateSigner(umi);
  await umi.rpc.airdrop(firstAssetOwner.publicKey, sol(10));
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const collection = await createCollection(umi, {
    payer: firstAssetOwner,
  });

  const asset = await createAsset(umi, {
    authority: firstAssetOwner,
    owner: firstAssetOwner,
    collection: collection.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  // move asset #1 twice as a delegate for collection
  await transferV1(umi, {
    authority: firstAssetOwner,
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: newOwner,
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await burnV1(umi, {
    payer: firstAssetOwner,
    asset: asset.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  const afterAsset = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});

test('it can remove permanent burn plugin if update authority', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
      }),
    ],
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.PermanentBurnDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    permanentBurnDelegate: undefined,
  });
});

test('it can add another plugin on asset with permanent burn plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'TransferDelegate',
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
