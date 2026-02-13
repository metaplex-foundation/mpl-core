import test from 'ava';
import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import {
  addGroupPlugin,
  createGroupV1,
  RelationshipKind,
  updateGroupPlugin,
} from '../src';
import {
  assertGroup,
  createGroup,
  createUmi,
  DEFAULT_GROUP,
} from './_setupRaw';

// Verify creation of a group
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

test('it can create a new group', async (t: any) => {
  const umi = await createUmi();
  const group = await createGroup(umi, {
    name: 'My Group',
  });

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    name: 'My Group',
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('it rejects creating a group with itself as a child relationship', async (t: any) => {
  const umi = await createUmi();
  const group = generateSigner(umi);

  await t.throwsAsync(
    createGroupV1(umi, {
      name: 'Self Child Group',
      uri: DEFAULT_GROUP.uri,
      group,
      payer: umi.identity,
      relationships: [
        { kind: RelationshipKind.ChildGroup, key: group.publicKey },
      ],
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: group.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'IncorrectAccount' }
  );
});

test('it rejects creating a group with itself as a parent relationship', async (t: any) => {
  const umi = await createUmi();
  const group = generateSigner(umi);

  await t.throwsAsync(
    createGroupV1(umi, {
      name: 'Self Parent Group',
      uri: DEFAULT_GROUP.uri,
      group,
      payer: umi.identity,
      relationships: [
        { kind: RelationshipKind.ParentGroup, key: group.publicKey },
      ],
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: group.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'IncorrectAccount' }
  );
});

test(
  'it preserves plugin metadata on linked child and parent groups',
  async (t: any) => {
    const umi = await createUmi();
    const childGroup = await createGroup(umi, { name: 'Child Group' });
    const parentGroup = await createGroup(umi, { name: 'Parent Group' });

    await addGroupPlugin(umi, {
      group: childGroup.publicKey,
      plugin: {
        type: 'Attributes',
        attributeList: [{ key: 'child-key', value: 'child-value' }],
      },
      authority: umi.identity,
      logWrapper: LOG_WRAPPER,
    }).sendAndConfirm(umi);

    await addGroupPlugin(umi, {
      group: parentGroup.publicKey,
      plugin: {
        type: 'Attributes',
        attributeList: [{ key: 'parent-key', value: 'parent-value' }],
      },
      authority: umi.identity,
      logWrapper: LOG_WRAPPER,
    }).sendAndConfirm(umi);

    const newGroup = generateSigner(umi);
    await createGroupV1(umi, {
      name: 'Group With Relations',
      uri: DEFAULT_GROUP.uri,
      group: newGroup,
      payer: umi.identity,
      relationships: [
        { kind: RelationshipKind.ChildGroup, key: childGroup.publicKey },
        { kind: RelationshipKind.ParentGroup, key: parentGroup.publicKey },
      ],
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: childGroup.publicKey },
        { isSigner: false, isWritable: true, pubkey: parentGroup.publicKey },
      ])
      .sendAndConfirm(umi);

    await assertGroup(t, umi, {
      group: childGroup.publicKey,
      updateAuthority: umi.identity.publicKey,
      parentGroups: [newGroup.publicKey],
    });
    await assertGroup(t, umi, {
      group: parentGroup.publicKey,
      updateAuthority: umi.identity.publicKey,
      groups: [newGroup.publicKey],
    });

    // These updates must continue to succeed after linking relationships.
    // If CreateGroupV1 corrupts plugin metadata, either call fails.
    await updateGroupPlugin(umi, {
      group: childGroup.publicKey,
      authority: umi.identity,
      logWrapper: LOG_WRAPPER,
      plugin: {
        type: 'Attributes',
        attributeList: [{ key: 'child-updated', value: 'child-updated-value' }],
      },
    }).sendAndConfirm(umi);

    await updateGroupPlugin(umi, {
      group: parentGroup.publicKey,
      authority: umi.identity,
      logWrapper: LOG_WRAPPER,
      plugin: {
        type: 'Attributes',
        attributeList: [{ key: 'parent-updated', value: 'parent-updated-value' }],
      },
    }).sendAndConfirm(umi);
  }
);
