import {
  PublicKey,
  Umi,
  publicKey,
  sol,
  subtractAmounts,
} from '@metaplex-foundation/umi';
import test from 'ava';

import {
  PluginType,
  addPluginV1,
  collect,
  createPlugin,
  pluginAuthorityPair,
  removePluginV1,
} from '../src';
import { createAsset, createUmi } from './_setup';

const hasCollectAmount = async (umi: Umi, address: PublicKey) => {
  const account = await umi.rpc.getAccount(address);
  if (account.exists) {
    const rent = await umi.rpc.getRent(account.data.length);
    const diff = account.lamports.basisPoints - rent.basisPoints;
    return diff === sol(0.0015).basisPoints;
  }
  return false;
};

test('it can create a new asset with collect amount', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );
});

test('it can add asset plugin with collect amount', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
    initAuthority: null,
  }).sendAndConfirm(umi);

  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );
});

test('it can add remove asset plugin with collect amount', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);
  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );
});

test('it can collect', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const balStart = await umi.rpc.getBalance(
    publicKey('8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v')
  );
  await collect(umi, {})
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);
  const balEnd = await umi.rpc.getBalance(
    publicKey('8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v')
  );
  t.is(await hasCollectAmount(umi, asset.publicKey), false);
  t.deepEqual(subtractAmounts(balEnd, balStart), sol(0.0015));
});
