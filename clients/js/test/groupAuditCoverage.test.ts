import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  addAssetsToGroup,
  addCollectionsToGroup,
  addGroupsToGroup,
  burn,
  burnCollection,
  closeGroup,
  createGroupV1,
  removeAssetsFromGroup,
  removeCollectionsFromGroup,
  removeGroupsFromGroup,
  RelationshipKind,
  updateGroup,
} from '../src';
import {
  assertAsset,
  assertBurned,
  assertCollection,
  assertGroup,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createGroup,
  createUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
} from './_setupRaw';

// =============================================================================
// GAP-01: Negative Authorization Tests
// =============================================================================

test('it rejects addAssetsToGroup when signer is not group authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const group = await createGroup(umi);
  const asset = await createAsset(umi);

  const builder = addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: attacker,
  }).addRemainingAccounts([
    { isSigner: false, isWritable: true, pubkey: asset.publicKey },
  ]);

  await t.throwsAsync(builder.sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

test('it rejects removeAssetsFromGroup when signer is not group authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const group = await createGroup(umi);
  const asset = await createAsset(umi);

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  const builder = removeAssetsFromGroup(umi, {
    group: group.publicKey,
    authority: attacker,
    assets: [asset.publicKey],
  }).addRemainingAccounts([
    { isSigner: false, isWritable: true, pubkey: asset.publicKey },
  ]);

  await t.throwsAsync(builder.sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

test('it rejects addCollectionsToGroup when signer is not group authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  const builder = addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: attacker,
  }).addRemainingAccounts([
    { isSigner: false, isWritable: true, pubkey: collection.publicKey },
  ]);

  await t.throwsAsync(builder.sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

test('it rejects removeCollectionsFromGroup when signer is not group authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  const builder = removeCollectionsFromGroup(umi, {
    group: group.publicKey,
    authority: attacker,
    collections: [collection.publicKey],
  }).addRemainingAccounts([
    { isSigner: false, isWritable: true, pubkey: collection.publicKey },
  ]);

  await t.throwsAsync(builder.sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

test('it rejects addGroupsToGroup when signer is not parent group authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const parent = await createGroup(umi, { name: 'parent' });
  const child = await createGroup(umi, { name: 'child' });

  const builder = addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: attacker,
  }).addRemainingAccounts([
    { isSigner: false, isWritable: true, pubkey: child.publicKey },
  ]);

  await t.throwsAsync(builder.sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

test('it rejects removeGroupsFromGroup when signer is not parent group authority', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const parent = await createGroup(umi, { name: 'parent' });
  const child = await createGroup(umi, { name: 'child' });

  await addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child.publicKey },
    ])
    .sendAndConfirm(umi);

  const builder = removeGroupsFromGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: attacker,
  }).addRemainingAccounts([
    { isSigner: false, isWritable: true, pubkey: child.publicKey },
  ]);

  await t.throwsAsync(builder.sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

// =============================================================================
// GAP-02: Edge Case Tests
// =============================================================================

test('it rejects adding an already-member asset (duplicate entry)', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi);

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(
    addAssetsToGroup(umi, {
      group: group.publicKey,
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: asset.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'DuplicateEntry' }
  );
});

test('it rejects adding an already-member collection (duplicate entry)', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(
    addCollectionsToGroup(umi, {
      group: group.publicKey,
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: collection.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'DuplicateEntry' }
  );
});

test('it rejects closing a group that still has child collections', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(
    closeGroup(umi, {
      group: group.publicKey,
    }).sendAndConfirm(umi),
    { name: 'GroupMustBeEmpty' }
  );
});

test('it rejects closing a group that still has child groups', async (t) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });
  const child = await createGroup(umi, { name: 'child' });

  await addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child.publicKey },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(
    closeGroup(umi, {
      group: parent.publicKey,
    }).sendAndConfirm(umi),
    { name: 'GroupMustBeEmpty' }
  );
});

test('it rejects closing a group that still has parent groups', async (t) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });
  const child = await createGroup(umi, { name: 'child' });

  await addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child.publicKey },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(
    closeGroup(umi, {
      group: child.publicKey,
    }).sendAndConfirm(umi),
    { name: 'GroupMustBeEmpty' }
  );
});

test('it can updateGroup with both name and URI simultaneously', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi, {
    name: 'original',
    uri: 'https://original.com',
  });

  await updateGroup(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    newName: 'updated',
    newUri: 'https://updated.com',
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'updated',
    uri: 'https://updated.com',
  });
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

test('it rejects removing a child group that is not linked', async (t) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });
  const unlinked = await createGroup(umi, { name: 'unlinked' });

  await t.throwsAsync(
    removeGroupsFromGroup(umi, {
      parentGroup: parent.publicKey,
      groups: [unlinked.publicKey],
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: unlinked.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'IncorrectAccount' }
  );
});

// =============================================================================
// GAP-03: Duplicate Remaining Accounts Tests
// =============================================================================

test('it rejects duplicate asset in remaining accounts for addAssetsToGroup', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi);

  await t.throwsAsync(
    addAssetsToGroup(umi, {
      group: group.publicKey,
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: asset.publicKey },
        { isSigner: false, isWritable: true, pubkey: asset.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'DuplicateEntry' }
  );
});

test('it rejects duplicate collection in remaining accounts for addCollectionsToGroup', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  await t.throwsAsync(
    addCollectionsToGroup(umi, {
      group: group.publicKey,
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: collection.publicKey },
        { isSigner: false, isWritable: true, pubkey: collection.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'DuplicateEntry' }
  );
});

test('it rejects duplicate child group in addGroupsToGroup', async (t) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });
  const child = await createGroup(umi, { name: 'child' });

  await t.throwsAsync(
    addGroupsToGroup(umi, {
      parentGroup: parent.publicKey,
      groups: [child.publicKey, child.publicKey],
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: child.publicKey },
        { isSigner: false, isWritable: true, pubkey: child.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'DuplicateEntry' }
  );
});

// =============================================================================
// GAP-04: Max Size / Nesting Boundaries
// =============================================================================

test.serial(
  'it rejects addGroupsToGroup when child group vector exceeds max size',
  async (t) => {
    const umi = await createUmi();
    const parent = await createGroup(umi, { name: 'parent-max-size' });

    // Keep this in sync with MAX_GROUP_VECTOR_SIZE in Rust state.
    const maxChildGroups = 256;

    for (let i = 0; i < maxChildGroups; i += 1) {
      const child = await createGroup(umi, { name: `child-${i}` });
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

    const overflowChild = await createGroup(umi, { name: 'child-overflow' });
    await t.throwsAsync(
      addGroupsToGroup(umi, {
        parentGroup: parent.publicKey,
        groups: [overflowChild.publicKey],
        authority: umi.identity,
      })
        .addRemainingAccounts([
          {
            isSigner: false,
            isWritable: true,
            pubkey: overflowChild.publicKey,
          },
        ])
        .sendAndConfirm(umi),
      { name: 'GroupVectorFull' }
    );
  }
);

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

// =============================================================================
// Additional Burn Consistency Coverage (C-02 fix verification)
// =============================================================================

test('it rejects burning an asset that belongs to a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi);

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    groups: {
      authority: { type: 'UpdateAuthority' },
      groups: [group.publicKey],
    },
  });

  await t.throwsAsync(burn(umi, { asset }).sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});

test('it allows burning an asset after removing it from all groups', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi);

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await removeAssetsFromGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
    assets: [asset.publicKey],
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await burn(umi, { asset }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('it rejects burning a collection that belongs to a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: {
      authority: { type: 'UpdateAuthority' },
      groups: [group.publicKey],
    },
  });

  await t.throwsAsync(
    burnCollection(umi, {
      collection: collection.publicKey,
      compressionProof: null,
    }).sendAndConfirm(umi),
    { name: 'InvalidAuthority' }
  );
});

test('it allows burning a collection after removing it from all groups', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection = await createCollection(umi);

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await removeCollectionsFromGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
    collections: [collection.publicKey],
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await burnCollection(umi, {
    collection: collection.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, collection.publicKey);
});

test('it rejects burning an asset in a collection that belongs to a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const { asset, collection } = await createAssetWithCollection(umi, {});

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(burn(umi, { asset, collection }).sendAndConfirm(umi), {
    name: 'InvalidAuthority',
  });
});
