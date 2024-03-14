import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  pluginAuthorityPair,
  burn,
  Key,
  getOwnerAuthority,
  transfer,
} from '../../../src';
import {
  createAsset,
  createUmi,
} from '../../_setup';
import { assertAccountExists, sol } from '@metaplex-foundation/umi';

test('it can burn an assets as an owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner: owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurn',
        authority: getOwnerAuthority(),
      }),
    ],
  });

  await burn(umi, {
    payer: owner,
    authority: owner,
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  const afterAsset = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});

test('it can burn an assets as a delegate', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const owner = generateSigner(umi);
    const newOwner = generateSigner(umi);
  
    const asset = await createAsset(umi, {
      owner: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentBurn',
          authority: getOwnerAuthority(),
        }),
      ],
    });

    await transfer(umi, {
        authority: owner,
        asset: asset.publicKey,
        newOwner: newOwner.publicKey,
    }).sendAndConfirm(umi);
  
    await burn(umi, {
        payer: owner,
        authority: owner,
        asset: asset.publicKey,
    }).sendAndConfirm(umi);
  
    const afterAsset = await umi.rpc.getAccount(asset.publicKey);
    t.true(afterAsset.exists);
    assertAccountExists(afterAsset);
    t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
    t.is(afterAsset.data.length, 1);
    t.is(afterAsset.data[0], Key.Uninitialized);
  });
