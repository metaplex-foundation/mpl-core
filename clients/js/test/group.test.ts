import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  addAssetsToGroup,
  addCollectionsToGroup,
  getGroupV1GpaBuilder,
  Key,
  removeAssetsFromGroup,
  removeCollectionsFromGroup,
} from '../src';
import { createAsset, createCollection, createGroup, createUmi } from './_setupRaw';

test('it can gpa fetch groups by updateAuthority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);

  await createGroup(umi, {
    name: 'group1',
    updateAuthority: updateAuthority.publicKey,
  });
  await createGroup(umi, {
    name: 'group2',
    updateAuthority: updateAuthority.publicKey,
  });
  await createGroup(umi, { name: 'group3' });

  const groups = await getGroupV1GpaBuilder(umi)
    .whereField('updateAuthority', updateAuthority.publicKey)
    .whereField('key', Key.GroupV1)
    .getDeserialized();
  const names = ['group1', 'group2'];

  t.is(groups.length, 2);
  t.assert(groups.every((g) => names.includes(g.name)));
  t.assert(
    groups.every((g) => g.updateAuthority === updateAuthority.publicKey)
  );
});

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
