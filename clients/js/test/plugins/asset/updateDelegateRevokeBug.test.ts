/**
 * Tests for PR 253: Operator precedence bug in UpdateDelegate validate_revoke_plugin_authority
 *
 * This test file validates a security vulnerability where the UpdateDelegate plugin's
 * validate_revoke_plugin_authority function has incorrect operator precedence. Due to
 * Rust's operator precedence (&&  binds tighter than ||), the condition:
 *
 *   A || B && C && D
 *
 * is evaluated as:
 *
 *   A || (B && C && D)
 *
 * instead of the intended:
 *
 *   (A || (B && C)) && D
 *
 * This means the manager check (plugin.manager() == Authority::UpdateAuthority) only
 * applies to the additional_delegates branch, not the main resolved_authorities branch.
 *
 * The result is that UpdateDelegate can revoke authority on owner-managed plugins
 * (like FreezeDelegate, TransferDelegate) when it should only be able to revoke
 * authority on UpdateAuthority-managed plugins.
 */

import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { approvePluginAuthority, revokePluginAuthority } from '../../../src';
import { DEFAULT_ASSET, assertAsset, createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';

/**
 * This test demonstrates the operator precedence bug in validate_revoke_plugin_authority.
 *
 * When the UpdateAuthority (who implicitly has UpdateDelegate powers via resolved_authorities)
 * tries to revoke authority on an owner-managed plugin (FreezeDelegate), it should fail
 * because FreezeDelegate is owner-managed, not UpdateAuthority-managed.
 *
 * Due to the bug, this operation incorrectly SUCCEEDS because the manager check
 * doesn't apply to the resolved_authorities branch.
 *
 * EXPECTED BEHAVIOR (after fix): This test should FAIL (throw NoApprovals)
 * CURRENT BEHAVIOR (with bug): This test PASSES (allows the revoke)
 */
test('it should NOT allow update authority to revoke authority on owner-managed plugins via UpdateDelegate', async (t) => {
  const umi = await createUmi();
  const freezeDelegateAuthority = generateSigner(umi);

  // Create an asset with:
  // 1. UpdateDelegate plugin (default authority: UpdateAuthority)
  // 2. FreezeDelegate plugin (default authority: Owner - an owner-managed plugin)
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
    ],
  });

  // Verify the asset was created with correct plugin authorities
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [],
    },
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: false,
    },
  });

  // Now as the owner, approve a new authority for FreezeDelegate
  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'FreezeDelegate' },
    newAuthority: {
      type: 'Address',
      address: freezeDelegateAuthority.publicKey,
    },
  }).sendAndConfirm(umi);

  // Verify the FreezeDelegate authority was changed
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [],
    },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: freezeDelegateAuthority.publicKey,
      },
      frozen: false,
    },
  });

  // Now, try to revoke authority on FreezeDelegate using the UpdateAuthority
  // (who has UpdateDelegate powers via resolved_authorities).
  //
  // THIS SHOULD FAIL because FreezeDelegate is an owner-managed plugin,
  // not an UpdateAuthority-managed plugin. UpdateDelegate should only be able
  // to revoke authority on UpdateAuthority-managed plugins.
  //
  // Due to the operator precedence bug, this incorrectly succeeds.
  const result = revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'FreezeDelegate' },
    // Using the default authority (UpdateAuthority/identity)
  }).sendAndConfirm(umi);

  // After the fix, this should throw NoApprovals
  // With the bug, this will pass (no error thrown)
  await t.throwsAsync(result, { name: 'NoApprovals' });
});

/**
 * This test is the counterpart - it verifies that UpdateDelegate CAN revoke
 * authority on UpdateAuthority-managed plugins. This should always work.
 */
test('it should allow update authority to revoke authority on UpdateAuthority-managed plugins via UpdateDelegate', async (t) => {
  const umi = await createUmi();
  const editionAuthority = generateSigner(umi);

  // Create an asset with:
  // 1. UpdateDelegate plugin (default authority: UpdateAuthority)
  // 2. Edition plugin (default authority: UpdateAuthority - an authority-managed plugin)
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
      {
        type: 'Edition',
        number: 1,
      },
    ],
  });

  // Approve a new authority for Edition
  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'Edition' },
    newAuthority: { type: 'Address', address: editionAuthority.publicKey },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [],
    },
    edition: {
      authority: { type: 'Address', address: editionAuthority.publicKey },
      number: 1,
    },
  });

  // Revoke authority on Edition using the UpdateAuthority
  // This SHOULD succeed because Edition is UpdateAuthority-managed
  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'Edition' },
  }).sendAndConfirm(umi);

  // Edition authority should be reverted to UpdateAuthority
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [],
    },
    edition: {
      authority: { type: 'UpdateAuthority' },
      number: 1,
    },
  });

  t.pass();
});

/**
 * Test the bug with TransferDelegate (another owner-managed plugin)
 */
test('it should NOT allow update authority to revoke authority on TransferDelegate via UpdateDelegate', async (t) => {
  const umi = await createUmi();
  const transferDelegateAuthority = generateSigner(umi);

  // Create an asset with UpdateDelegate and TransferDelegate
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
      },
      {
        type: 'TransferDelegate',
      },
    ],
  });

  // Approve a new authority for TransferDelegate (as owner)
  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'TransferDelegate' },
    newAuthority: {
      type: 'Address',
      address: transferDelegateAuthority.publicKey,
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: { type: 'UpdateAuthority' },
      additionalDelegates: [],
    },
    transferDelegate: {
      authority: {
        type: 'Address',
        address: transferDelegateAuthority.publicKey,
      },
    },
  });

  // Try to revoke authority on TransferDelegate using the UpdateAuthority
  // This SHOULD FAIL because TransferDelegate is owner-managed
  const result = revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'TransferDelegate' },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
});

/**
 * Test that a separate UpdateDelegate authority (not the UpdateAuthority itself)
 * also cannot revoke authority on owner-managed plugins.
 *
 * This tests the same bug but through a delegated UpdateDelegate authority.
 */
test('it should NOT allow delegated update delegate to revoke authority on owner-managed plugins', async (t) => {
  const umi = await createUmi();
  const updateDelegateAuthority = generateSigner(umi);
  const freezeDelegateAuthority = generateSigner(umi);

  // Create an asset with:
  // 1. UpdateDelegate plugin with a separate delegated authority
  // 2. FreezeDelegate plugin
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'UpdateDelegate',
        additionalDelegates: [],
        authority: {
          type: 'Address',
          address: updateDelegateAuthority.publicKey,
        },
      },
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
    ],
  });

  // As owner, approve a new authority for FreezeDelegate
  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'FreezeDelegate' },
    newAuthority: {
      type: 'Address',
      address: freezeDelegateAuthority.publicKey,
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    updateDelegate: {
      authority: {
        type: 'Address',
        address: updateDelegateAuthority.publicKey,
      },
      additionalDelegates: [],
    },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: freezeDelegateAuthority.publicKey,
      },
      frozen: false,
    },
  });

  // Try to revoke authority on FreezeDelegate using the delegated UpdateDelegate
  // This SHOULD FAIL because FreezeDelegate is owner-managed
  const result = revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: { type: 'FreezeDelegate' },
    authority: updateDelegateAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
});
