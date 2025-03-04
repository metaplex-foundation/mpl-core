import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { transferV1 } from '../../../src';
import { assertAsset, createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';

test('it can transfer an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'TransferCount',
        count: 0,
      },
    ],
  });
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferCount: { count: 0n, authority: { type: 'UpdateAuthority' } },
  });

  const tx = await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  console.log((await umi.rpc.getTransaction(tx.signature))?.meta?.logs);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferCount: { count: 1n, authority: { type: 'UpdateAuthority' } },
  });
});
