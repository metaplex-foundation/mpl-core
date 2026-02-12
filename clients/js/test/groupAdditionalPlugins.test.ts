import { publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { addGroupPlugin, removeGroupPlugin, updateGroupPlugin } from '../src';
import {
  DEFAULT_GROUP,
  assertGroup,
  createGroup,
  createUmi,
} from './_setupRaw';

// Re-use the same Noop log wrapper program used across existing plugin tests.
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Group Plugins – Autograph
// -----------------------------------------------------------------------------

test('it can add, update, and remove an Autograph plugin on a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  // ---------------------------------------------------------------------------
  // 1. Add plugin with a single signature.
  // ---------------------------------------------------------------------------
  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'Initial signature',
        },
      ],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // Validate the group still matches our expectations (plugin data is checked in
  // dedicated plugin unit tests – here we merely ensure the instruction
  // executed successfully and the group account remains consistent).
  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  // ---------------------------------------------------------------------------
  // 2. Update plugin to include a second signature.
  // ---------------------------------------------------------------------------
  await updateGroupPlugin(umi, {
    group: group.publicKey,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    plugin: {
      type: 'Autograph',
      signatures: [
        { address: umi.identity.publicKey, message: 'sig-0' },
        { address: umi.identity.publicKey, message: 'sig-1' },
      ],
    },
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  // ---------------------------------------------------------------------------
  // 3. Remove the plugin entirely.
  // ---------------------------------------------------------------------------
  await removeGroupPlugin(umi, {
    group: group.publicKey,
    plugin: { type: 'Autograph' },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

// -----------------------------------------------------------------------------
// Group Plugins – VerifiedCreators
// -----------------------------------------------------------------------------

test('it can add and remove a VerifiedCreators plugin on a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  // Add VerifiedCreators plugin with a single verified creator.
  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
      ],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  // Remove the plugin.
  await removeGroupPlugin(umi, {
    group: group.publicKey,
    plugin: { type: 'VerifiedCreators' },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});
