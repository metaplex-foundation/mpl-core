import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getCreateV1Instruction,
  fetchAssetV1,
  findAssetSignerPda,
  Key,
} from '../../../src';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can create asset with FreezeExecute plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetSigner = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetSigner,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        plugin: {
          __kind: 'FreezeExecute',
          fields: [{ frozen: true }],
        },
        authority: { __option: 'None' },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetSigner, payer]
  );

  const asset = await fetchAssetV1(rpc, assetSigner.address);

  t.is(asset.data.key, Key.AssetV1);
  t.is(asset.data.owner, payer.address);
  t.is(asset.data.name, 'Test Asset');
});

test('it can derive asset signer PDA', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetSigner = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetSigner,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        plugin: {
          __kind: 'FreezeExecute',
          fields: [{ frozen: true }],
        },
        authority: { __option: 'None' },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetSigner, payer]
  );

  const asset = await fetchAssetV1(rpc, assetSigner.address);

  // Verify we can derive the asset signer PDA
  const [assetSignerPda] = await findAssetSignerPda({
    asset: asset.address,
  });

  t.truthy(assetSignerPda);
  t.is(typeof assetSignerPda, 'string');
});
