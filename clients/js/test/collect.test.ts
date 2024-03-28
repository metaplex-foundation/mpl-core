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
  burnV1,
  collect,
  createPlugin,
  pluginAuthorityPair,
  removePluginV1,
} from '../src';
import { createAsset, createUmi } from './_setup';

const recipient1 = publicKey('8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v')
const recipient2 = publicKey('MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt')

test.before(async () => {
  const umi = await createUmi();
  await umi.rpc.airdrop(recipient1, sol(0.1));
  await umi.rpc.airdrop(recipient2, sol(0.1));
})

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
  const umi = await createUmi();
  const asset = await createAsset(umi);

  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );
});

test('it can add asset plugin with collect amount', async (t) => {
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
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
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

test.serial('it can collect', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const balStart1 = await umi.rpc.getBalance(recipient1);
  const balStart2 = await umi.rpc.getBalance(recipient2);
  await collect(umi, {})
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);
  const balEnd1 = await umi.rpc.getBalance(
    recipient1
  );
  const balEnd2 = await umi.rpc.getBalance(
    recipient2
  );
  t.is(await hasCollectAmount(umi, asset.publicKey), false);
  t.deepEqual(subtractAmounts(balEnd1, balStart1), sol(0.0015/2));
  t.deepEqual(subtractAmounts(balEnd2, balStart2), sol(0.0015/2));
});

test.serial('it can collect burned asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const balStart1 = await umi.rpc.getBalance(recipient1);
  const balStart2 = await umi.rpc.getBalance(recipient2);

  await burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  await collect(umi, {})
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);
  const balEnd1 = await umi.rpc.getBalance(
    recipient1
  );
  const balEnd2 = await umi.rpc.getBalance(
    recipient2
  );
  t.is(await hasCollectAmount(umi, asset.publicKey), false);
  t.deepEqual(subtractAmounts(balEnd1, balStart1), sol(0.0015/2));
  t.deepEqual(subtractAmounts(balEnd2, balStart2), sol(0.0015/2));
})