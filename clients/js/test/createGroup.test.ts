import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { addGroupsToGroup, createGroupV1, RelationshipKind } from '../src';
import {
  assertAsset,
  assertCollection,
  assertGroup,
  createAsset,
  createAssetWithCollection,
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

test('it allows collection authority to link collection-managed assets in createGroupV1', async (t) => {
  const umi = await createUmi();
  const sharedAuthority = generateSigner(umi);
  const group = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(umi, {
    updateAuthority: sharedAuthority,
  });

  await createGroupV1(umi, {
    name: 'collection-managed-asset-group',
    uri: 'https://example.com/collection-managed-asset-group',
    group,
    payer: umi.identity,
    updateAuthority: sharedAuthority,
    relationships: [{ kind: RelationshipKind.Asset, key: asset.publicKey }],
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
      // Supplemental collection account used for collection-authority resolution.
      { isSigner: false, isWritable: false, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: sharedAuthority.publicKey,
    name: 'collection-managed-asset-group',
    uri: 'https://example.com/collection-managed-asset-group',
    assets: [asset.publicKey],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: collection.publicKey,
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

  const error = await t.throwsAsync(
    createGroupV1(umi, {
      name: 'too-many-parents',
      uri: 'https://example.com/too-many-parents',
      group,
      payer: umi.identity,
      relationships: tooManyParentRelationships,
    }).sendAndConfirm(umi)
  );

  t.truthy(error);
  t.regex((error as Error).message, /Group nesting depth exceeded/);
});

test.serial(
  'it rejects createGroupV1 when linking a child group that is already at max nesting depth',
  async (t) => {
    const umi = await createUmi();
    const child = await createGroup(umi, { name: 'max-depth-child' });

    const maxDepth = 8;
    for (let i = 0; i < maxDepth; i += 1) {
      const parent = await createGroup(umi, { name: `depth-parent-${i}` });
      await addGroupsToGroup(umi, {
        parentGroup: parent.publicKey,
        groups: [child.publicKey],
        authority: umi.identity,
      })
        .addRemainingAccounts([
          { isSigner: false, isWritable: true, pubkey: child.publicKey },
        ])
        .sendAndConfirm(umi);
    }

    const newGroup = generateSigner(umi);
    const error = await t.throwsAsync(
      createGroupV1(umi, {
        name: 'would-exceed-child-depth',
        uri: 'https://example.com/would-exceed-child-depth',
        group: newGroup,
        payer: umi.identity,
        relationships: [
          { kind: RelationshipKind.ChildGroup, key: child.publicKey },
        ],
      })
        .addRemainingAccounts([
          { isSigner: false, isWritable: true, pubkey: child.publicKey },
        ])
        .sendAndConfirm(umi)
    );

    t.truthy(error);
    t.regex((error as Error).message, /Group nesting depth exceeded/);
  }
);
