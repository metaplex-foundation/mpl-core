import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';

import {
  approvePluginAuthorityV1,
  create,
  createPlugin,
  execute,
  fetchAssetV1,
  findAssetSignerPda,
  PluginType,
  removePluginV1,
  revokePluginAuthorityV1,
  updatePluginV1,
} from '../../../src';
import { assertAsset, createUmi, DEFAULT_ASSET } from '../../_setupRaw';

test('it cannot remove FreezeExecute while frozen', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  // 1. Create an asset with FreezeExecute frozen.
  //    Owner and updateAuthority are distinct.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [{ type: 'FreezeExecute', frozen: true }],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });

  // 2. Owner removes FreezeExecute while frozen.
  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeExecute,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot revoke FreezeExecute as the owner while frozen', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  // 1. Create an asset with FreezeExecute frozen.
  //    Owner and updateAuthority are distinct.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [{ type: 'FreezeExecute', frozen: true }],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });

  // 2. Owner revokes FreezeExecute while frozen.
  const result = revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeExecute,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot approve a new authority for FreezeExecute as the owner while frozen', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const assetSigner = generateSigner(umi);
  const delegate = generateSigner(umi);

  // 1. Create an asset with FreezeExecute frozen.
  //    Owner and updateAuthority are distinct.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [{ type: 'FreezeExecute', frozen: true }],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });

  // 2. Owner approves a new authority for FreezeExecute while frozen.
  const result = approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeExecute,
    newAuthority: { __kind: 'Address', address: delegate.publicKey },
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('Protocol-delegated FreezeExecute cannot be removed by owner despite freeze', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const protocol = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  // 1. Create asset with owner != updateAuthority.
  //    FreezeExecute is delegated to a protocol address and frozen.
  //    This simulates a protocol that locks Execute to guard PDA funds.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'FreezeExecute',
        frozen: true,
        authority: { type: 'Address', address: protocol.publicKey },
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));
  const [assetSignerPda] = findAssetSignerPda(umi, { asset: asset.publicKey });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Address', address: protocol.publicKey },
      frozen: true,
    },
  });

  // 2. Fund the PDA (simulating protocol-deposited funds).
  await transferSol(umi, {
    source: umi.identity,
    destination: publicKey(assetSignerPda),
    amount: sol(0.5),
  }).sendAndConfirm(umi);

  // 3. Confirm Execute is blocked.
  const recipient = generateSigner(umi);
  const blockedExec = execute(umi, {
    asset,
    authority: owner,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSignerPda)),
      destination: recipient.publicKey,
      amount: sol(0.1),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(blockedExec, { name: 'InvalidAuthority' });

  // 4. Owner removes the protocol-delegated FreezeExecute while frozen.
  //    The protocol delegated this plugin and froze it — only the protocol
  //    should be able to unfreeze/remove. But owner-managed removal ignores
  //    the frozen state.
  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeExecute,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('owner cannot unfreeze delegate-frozen FreezeExecute', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const delegate = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  // 1. Create asset with FreezeExecute delegated to a third party and frozen.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'FreezeExecute',
        frozen: true,
        authority: { type: 'Address', address: delegate.publicKey },
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Address', address: delegate.publicKey },
      frozen: true,
    },
  });

  // 2. Owner attempts to unfreeze — should be rejected (not the delegate).
  const unfreezeResult = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeExecute', data: { frozen: false } }),
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(unfreezeResult, { name: 'NoApprovals' });

  // Plugin is still frozen.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Address', address: delegate.publicKey },
      frozen: true,
    },
  });
});

test('delegate can unfreeze FreezeExecute', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const delegate = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  // 1. Create asset with FreezeExecute delegated and frozen.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'FreezeExecute',
        frozen: true,
        authority: { type: 'Address', address: delegate.publicKey },
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  // 2. Delegate unfreezes — should succeed.
  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeExecute', data: { frozen: false } }),
    authority: delegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freezeExecute: {
      authority: { type: 'Address', address: delegate.publicKey },
      frozen: false,
    },
  });
});

test('owner can remove FreezeExecute after delegate unfreezes it', async (t) => {
  const umi = await createUmi();
  const owner = umi.identity;
  const updateAuth = generateSigner(umi);
  const delegate = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  // 1. Create asset with FreezeExecute delegated and frozen.
  await create(umi, {
    asset: assetSigner,
    owner: owner.publicKey,
    updateAuthority: updateAuth.publicKey,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    plugins: [
      {
        type: 'FreezeExecute',
        frozen: true,
        authority: { type: 'Address', address: delegate.publicKey },
      },
    ],
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, publicKey(assetSigner));

  // 2. Removal while frozen — should fail.
  const blockedResult = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeExecute,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(blockedResult, { name: 'InvalidAuthority' });

  // 3. Delegate unfreezes.
  await updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeExecute', data: { frozen: false } }),
    authority: delegate,
  }).sendAndConfirm(umi);

  // 4. Owner removes now that it's unfrozen — should succeed.
  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeExecute,
    authority: owner,
  }).sendAndConfirm(umi);

  const assetAfter = await fetchAssetV1(umi, asset.publicKey);
  t.is(assetAfter.freezeExecute, undefined);
});
