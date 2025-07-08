import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  addGroupPlugin,
  approveGroupPluginAuthority,
  revokeGroupPluginAuthority,
  updateGroupPlugin,
} from '../src';
import {
  assertGroup,
  createGroup,
  createUmi,
  DEFAULT_GROUP,
} from './_setupRaw';

// Same log-wrapper constant used in existing tests.
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Approve & Revoke Plugin Authority
// -----------------------------------------------------------------------------

test('it can approve and subsequently revoke a plugin authority on a group', async (t) => {
  // ---------------------------------------------------------------------------
  // Setup: create a group with an Attributes plugin whose authority is None.
  // ---------------------------------------------------------------------------
  const umi = await createUmi();
  const group = await createGroup(umi);

  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'init', value: 'value' }],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // ---------------------------------------------------------------------------
  // Approve: delegate plugin authority to a new signer.
  // ---------------------------------------------------------------------------
  const newAuthority = generateSigner(umi);

  await approveGroupPluginAuthority(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    plugin: { type: 'Attributes' },
    newAuthority: { type: 'Address', address: newAuthority.publicKey },
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // The new authority should now be able to update the plugin.
  await updateGroupPlugin(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'k1', value: 'v1' }],
    },
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  // ---------------------------------------------------------------------------
  // Revoke: remove the dedicated authority and ensure it can no longer act.
  // ---------------------------------------------------------------------------
  await revokeGroupPluginAuthority(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    plugin: { type: 'Attributes' },
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  // Attempting an update with the revoked authority should now fail.
  await t.throwsAsync(
    updateGroupPlugin(umi, {
      group: group.publicKey,
      payer: umi.identity,
      authority: newAuthority,
      logWrapper: LOG_WRAPPER,
      plugin: {
        type: 'Attributes',
        attributeList: [{ key: 'fail', value: 'fail' }],
      },
    }).sendAndConfirm(umi)
  );
});
