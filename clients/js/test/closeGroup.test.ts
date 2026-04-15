import test from 'ava';
import {
  addAssetsToGroup,
  addCollectionsToGroup,
  addGroupsToGroup,
  closeGroup,
} from '../src';
import {
  assertBurned,
  createAsset,
  createCollection,
  createGroup,
  createUmi,
} from './_setupRaw';

test('it can close a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  await closeGroup(umi, {
    group: group.publicKey,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, group.publicKey);
});

test('it cannot close a group with child assets', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);
  const asset = await createAsset(umi, {});

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: umi.identity,
  })
    .addRemainingAccounts([
      {
        isSigner: false,
        isWritable: true,
        pubkey: asset.publicKey,
      },
    ])
    .sendAndConfirm(umi);

  await t.throwsAsync(
    closeGroup(umi, {
      group: group.publicKey,
    }).sendAndConfirm(umi),
    { name: 'GroupMustBeEmpty' }
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
