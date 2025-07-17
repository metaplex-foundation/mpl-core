import test from 'ava';
import {
  addAssetsToGroup,
  addCollectionsToGroup,
  addGroupsToGroup,
  removeAssetsFromGroup,
  removeCollectionsFromGroup,
  removeGroupsFromGroup,
} from '../src';
import {
  assertGroup,
  createAsset,
  createCollection,
  createGroup,
  createUmi,
} from './_setupRaw';

test('it can add and remove assets from a group', async (t) => {
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

test('it can add and remove collections from a group', async (t) => {
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

test('it can add and remove child groups to a parent group', async (t) => {
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
