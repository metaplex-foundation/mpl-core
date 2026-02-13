import test from 'ava';
import { publicKey } from '@metaplex-foundation/umi';
import {
  addCollectionPlugin,
  addGroupPlugin,
  addPlugin,
  addAssetsToGroup,
  addCollectionsToGroup,
  addGroupsToGroup,
  removeAssetsFromGroup,
  removeCollectionsFromGroup,
  removeGroupsFromGroup,
} from '../src';
import {
  assertAsset,
  assertCollection,
  assertGroup,
  createAsset,
  createCollection,
  createGroup,
  createUmi,
} from './_setupRaw';

const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

test('it can add and remove assets from a group', async (t: any) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset1 = await createAsset(umi, {});
  const asset2 = await createAsset(umi, {});

  // Add assets
  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      {
        isSigner: false,
        isWritable: true,
        pubkey: asset1.publicKey,
      },
      {
        isSigner: false,
        isWritable: true,
        pubkey: asset2.publicKey,
      },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    assets: [asset1.publicKey, asset2.publicKey],
  });

  // Remove asset1
  await removeAssetsFromGroup(umi, {
    group: group.publicKey,
    assets: [asset1.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset1.publicKey,
    })
    .sendAndConfirm(umi);

  // Assert only asset2 remains
  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    assets: [asset2.publicKey],
  });

  t.pass();
});

test('it preserves trailing asset plugins when removing a group membership', async (t: any) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi, {});

  // Step 1: add the asset to the group so the internal Groups plugin exists.
  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);

  // Step 2: append a regular plugin after Groups in account layout.
  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'asset-key', value: 'asset-value' }],
    },
  }).sendAndConfirm(umi);

  // Step 3: remove membership and ensure trailing plugin data remains intact.
  await removeAssetsFromGroup(umi, {
    group: group.publicKey,
    assets: [asset.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    assets: [],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      attributeList: [{ key: 'asset-key', value: 'asset-value' }],
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can add and remove collections from a group', async (t: any) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection1 = await createCollection(umi, {});
  const collection2 = await createCollection(umi, {});

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      {
        isSigner: false,
        isWritable: true,
        pubkey: collection1.publicKey,
      },
      {
        isSigner: false,
        isWritable: true,
        pubkey: collection2.publicKey,
      },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    collections: [collection1.publicKey, collection2.publicKey],
  });

  // Remove collection1
  await removeCollectionsFromGroup(umi, {
    group: group.publicKey,
    collections: [collection1.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: collection1.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    collections: [collection2.publicKey],
  });
});

test('it preserves trailing collection plugins when removing a group membership', async (t: any) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const collection = await createCollection(umi, {});

  // Step 1: add collection to group so internal Groups plugin exists.
  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: collection.publicKey,
    })
    .sendAndConfirm(umi);

  // Step 2: append a regular collection plugin after Groups.
  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'collection-key', value: 'collection-value' }],
    },
  }).sendAndConfirm(umi);

  // Step 3: remove membership and validate trailing plugin integrity.
  await removeCollectionsFromGroup(umi, {
    group: group.publicKey,
    collections: [collection.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: collection.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    collections: [],
  });

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    attributes: {
      attributeList: [{ key: 'collection-key', value: 'collection-value' }],
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it preserves group plugins when adding/removing assets and collections', async (t: any) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi, {});
  const collection = await createCollection(umi, {});

  const expectedGroupAttributes = {
    authority: { type: 'UpdateAuthority' as const },
    attributeList: [{ key: 'group-key', value: 'group-value' }],
  };

  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: expectedGroupAttributes.attributeList,
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    assets: [asset.publicKey],
    attributes: expectedGroupAttributes,
  });

  await removeAssetsFromGroup(umi, {
    group: group.publicKey,
    assets: [asset.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: asset.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    assets: [],
    attributes: expectedGroupAttributes,
  });

  await addCollectionsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: collection.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    collections: [collection.publicKey],
    attributes: expectedGroupAttributes,
  });

  await removeCollectionsFromGroup(umi, {
    group: group.publicKey,
    collections: [collection.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: collection.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
    collections: [],
    attributes: expectedGroupAttributes,
  });
});

test('it preserves parent/child group plugins when linking and unlinking groups', async (t: any) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent-with-plugin' });
  const child = await createGroup(umi, { name: 'child-with-plugin' });

  const parentAttributes = {
    authority: { type: 'UpdateAuthority' as const },
    attributeList: [{ key: 'parent-key', value: 'parent-value' }],
  };
  const childAttributes = {
    authority: { type: 'UpdateAuthority' as const },
    attributeList: [{ key: 'child-key', value: 'child-value' }],
  };

  await addGroupPlugin(umi, {
    group: parent.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: parentAttributes.attributeList,
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await addGroupPlugin(umi, {
    group: child.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: childAttributes.attributeList,
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: child.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [child.publicKey],
    attributes: parentAttributes,
  });
  await assertGroup(t, umi, {
    group: child.publicKey,
    updateAuthority: umi.identity.publicKey,
    parentGroups: [parent.publicKey],
    attributes: childAttributes,
  });

  await removeGroupsFromGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts({
      isSigner: false,
      isWritable: true,
      pubkey: child.publicKey,
    })
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [],
    attributes: parentAttributes,
  });
  await assertGroup(t, umi, {
    group: child.publicKey,
    updateAuthority: umi.identity.publicKey,
    parentGroups: [],
    attributes: childAttributes,
  });
});

test('it can add and remove child groups to a parent group', async (t: any) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });
  const child1 = await createGroup(umi, { name: 'child1' });
  const child2 = await createGroup(umi, { name: 'child2' });

  await addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child1.publicKey, child2.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child1.publicKey },
      { isSigner: false, isWritable: true, pubkey: child2.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [child1.publicKey, child2.publicKey],
  });

  await removeGroupsFromGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child1.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child1.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [child2.publicKey],
  });

  t.pass();
});
