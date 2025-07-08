import { publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  addGroupPlugin,
  removeGroupPlugin,
  updateGroup,
  updateGroupPlugin,
} from '../src';
import {
  DEFAULT_GROUP,
  assertGroup,
  createGroup,
  createUmi,
} from './_setupRaw';

const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Group Plugins – Attributes
// -----------------------------------------------------------------------------

test('it can add attributes plugin to a group', async (t) => {
  // Given a Umi instance and a freshly created group.
  const umi = await createUmi();
  const group = await createGroup(umi);

  // When we add an Attributes plugin with an empty attribute list.
  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'init', value: 'value' }],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // Then the group account should reflect the new plugin.
  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it can update an attributes plugin on a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  // Add initial plugin with empty list.
  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'init', value: 'value' }],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // Update the plugin with two attributes.
  await updateGroupPlugin(umi, {
    group: group.publicKey,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    plugin: {
      type: 'Attributes',
      attributeList: [
        { key: 'key0', value: 'value0' },
        { key: 'key1', value: 'value1' },
      ],
    },
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it can remove an attributes plugin from a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  // Add plugin first.
  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'init', value: 'value' }],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // Remove the plugin.
  await removeGroupPlugin(umi, {
    group: group.publicKey,
    plugin: { type: 'Attributes' },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // The plugin should no longer be present.
  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

// -----------------------------------------------------------------------------
// Group Update – Name & URI
// -----------------------------------------------------------------------------

test("it can update a group's name and URI", async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  const NEW_NAME = 'Updated Group';
  const NEW_URI = 'https://example.com/updated-group';

  await updateGroup(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    newName: NEW_NAME,
    newUri: NEW_URI,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    name: NEW_NAME,
    uri: NEW_URI,
    updateAuthority: umi.identity.publicKey,
  });
});
