import { PublicKey, Umi, sol } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  PluginType,
  addPlugin,
  plugin,
  pluginAuthorityPair,
  removePlugin,
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

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: true }]),
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
        type: 'Freeze',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );

  await removePlugin(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Freeze,
  }).sendAndConfirm(umi);
  t.assert(
    await hasCollectAmount(umi, asset.publicKey),
    'Collect amount not found'
  );
});
