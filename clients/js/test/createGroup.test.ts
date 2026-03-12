import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { createGroupV1, RelationshipKind } from '../src';
import {
  assertAsset,
  assertCollection,
  assertGroup,
  createAsset,
  createCollection,
  createGroup,
  createUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  DEFAULT_GROUP,
} from './_setupRaw';

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

test('it can createGroupV1 with all four relationship kinds in one call', async (t) => {
  const umi = await createUmi();
  const group = generateSigner(umi);
  const collection = await createCollection(umi);
  const child = await createGroup(umi, { name: 'child' });
  const parent = await createGroup(umi, { name: 'parent' });
  const asset = await createAsset(umi);

  await createGroupV1(umi, {
    name: 'all-relationships',
    uri: 'https://example.com/all-relationships',
    group,
    payer: umi.identity,
    relationships: [
      { kind: RelationshipKind.Collection, key: collection.publicKey },
      { kind: RelationshipKind.ChildGroup, key: child.publicKey },
      { kind: RelationshipKind.ParentGroup, key: parent.publicKey },
      { kind: RelationshipKind.Asset, key: asset.publicKey },
    ],
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
      { isSigner: false, isWritable: true, pubkey: child.publicKey },
      { isSigner: false, isWritable: true, pubkey: parent.publicKey },
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'all-relationships',
    uri: 'https://example.com/all-relationships',
    collections: [collection.publicKey],
    groups: [child.publicKey],
    parentGroups: [parent.publicKey],
    assets: [asset.publicKey],
  });

  await assertGroup(t, umi, {
    group: child.publicKey,
    updateAuthority: umi.identity.publicKey,
    parentGroups: [group.publicKey],
  });

  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [group.publicKey],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: {
      authority: { type: 'UpdateAuthority' },
      groups: [group.publicKey],
    },
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Address',
      address: umi.identity.publicKey,
    },
    groups: {
      authority: { type: 'UpdateAuthority' },
      groups: [group.publicKey],
    },
  });
});

test('it rejects createGroupV1 when parent relationships exceed nesting depth', async (t) => {
  const umi = await createUmi();
  const group = generateSigner(umi);

  const tooManyParentRelationships = Array.from({ length: 9 }, () => ({
    kind: RelationshipKind.ParentGroup,
    key: generateSigner(umi).publicKey,
  }));

  await t.throwsAsync(
    createGroupV1(umi, {
      name: 'too-many-parents',
      uri: 'https://example.com/too-many-parents',
      group,
      payer: umi.identity,
      relationships: tooManyParentRelationships,
    }).sendAndConfirm(umi),
    { name: 'GroupNestingDepthExceeded' }
  );
});
