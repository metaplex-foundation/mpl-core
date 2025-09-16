import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';

import {
  addPluginV1,
  burnV1,
  create,
  createPlugin,
  execute,
  fetchAssetV1,
  findAssetSignerPda,
  PluginType,
  removePluginV1,
  transferV1,
  updatePluginV1,
} from '../../../src';
import {
  assertAsset,
  assertBurned,
  createAsset,
  createUmi,
  DEFAULT_ASSET,
} from '../../_setupRaw';

test('it can freeze and unfreeze execute with PermanentFreezeExecute', async (t) => {
  // Given a Umi instance
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(1));

  // Create an asset with PermanentFreezeExecute plugin (frozen by default)
  const assetSigner = generateSigner(umi);

  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Fund the asset signer PDA
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // Attempt Execute → should fail because plugin is frozen
  const recipient = generateSigner(umi);

  const execResult = execute(umi, {
    asset,
    payer: owner,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult, { name: 'InvalidAuthority' });

  // Unfreeze the execute with update authority
  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });

  // Now execute should succeed
  await execute(umi, {
    asset,
    payer: owner,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  // Verify recipient received the SOL
  const recipientBalance = await umi.rpc.getBalance(recipient.publicKey);
  t.true(recipientBalance.basisPoints >= sol(0.1).basisPoints);
});

test('it cannot add PermanentFreezeExecute after creation', async (t) => {
  // Given a Umi instance and a new signer
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, { owner });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: true },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: undefined,
  });
});

test('PermanentFreezeExecute persists after transfer to new owner and remains frozen', async (t) => {
  // Given a Umi instance
  const umi = await createUmi();
  const originalOwner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // Create an asset with PermanentFreezeExecute plugin (frozen)
  const assetSigner = generateSigner(umi);

  await create(umi, {
    asset: assetSigner,
    owner: originalOwner.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  // Verify initial state
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: originalOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Fund the asset signer PDA
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // Transfer the asset to a new owner
  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: originalOwner,
  }).sendAndConfirm(umi);

  // Verify the asset has been transferred
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  // Verify execute is still blocked with new owner
  const recipient = generateSigner(umi);

  const execResult = execute(umi, {
    asset,
    payer: newOwner,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult, { name: 'InvalidAuthority' });

  // Verify that only update authority (not the new owner) can unfreeze
  const unfreezeByOwner = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
    authority: newOwner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(unfreezeByOwner, { name: 'NoApprovals' });

  // Update authority can still unfreeze
  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'PermanentFreezeExecute',
      data: { frozen: false },
    }),
    authority: umi.identity,
  }).sendAndConfirm(umi);

  // Verify unfrozen
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot remove PermanentFreezeExecute plugin if frozen', async (t) => {
  const umi = await createUmi();

  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  }).sendAndConfirm(umi);
  const asset = await fetchAssetV1(umi, assetSigner.publicKey);

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it can remove PermanentFreezeExecute plugin if unfrozen', async (t) => {
  const umi = await createUmi();

  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: false,
      },
    ],
  }).sendAndConfirm(umi);
  const asset = await fetchAssetV1(umi, assetSigner.publicKey);

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.PermanentFreezeExecute,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: undefined,
  });
});

test('it can add other plugins alongside PermanentFreezeExecute', async (t) => {
  const umi = await createUmi();

  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: false,
      },
    ],
  }).sendAndConfirm(umi);
  const asset = await fetchAssetV1(umi, assetSigner.publicKey);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'TransferDelegate',
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeExecute: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('PermanentFreezeExecute blocks execute but allows burn', async (t) => {
  // Given a Umi instance
  const umi = await createUmi();
  const owner = generateSigner(umi);

  // Create an asset with PermanentFreezeExecute plugin (frozen)
  const assetSigner = generateSigner(umi);

  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'PermanentFreezeExecute',
        frozen: true,
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  // Fund the asset signer PDA
  const [assetSignerPda] = findAssetSignerPda(umi, {
    asset: asset.publicKey,
  });

  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // Verify execute is blocked
  const recipient = generateSigner(umi);
  const execResult = execute(umi, {
    asset,
    payer: owner,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(execResult, { name: 'InvalidAuthority' });

  // But burn should still work
  const balanceBefore = await umi.rpc.getBalance(owner.publicKey);

  await burnV1(umi, {
    asset: asset.publicKey,
    payer: owner,
  }).sendAndConfirm(umi);

  // Assert the asset account is burned
  await assertBurned(t, umi, asset.publicKey);

  const balanceAfter = await umi.rpc.getBalance(owner.publicKey);
  t.true(
    balanceAfter.basisPoints > balanceBefore.basisPoints,
    'Owner balance did not increase after burn refund'
  );
});
