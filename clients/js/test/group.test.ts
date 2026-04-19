import { generateSigner, PublicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  addAssetsToGroup,
  addCollectionsToGroup,
  getGroupV1GpaBuilder,
  Key,
  removeAssetsFromGroup,
  removeCollectionsFromGroup,
} from '../src';
import {
  assertGroup,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createGroup,
  createUmi,
} from './_setupRaw';

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

test('it allows collection update authority to add collection-managed assets to a group', async (t) => {
  const umi = await createUmi();
  const sharedAuthority = generateSigner(umi);

  const group = await createGroup(umi, { updateAuthority: sharedAuthority });
  const { asset, collection } = await createAssetWithCollection(umi, {
    updateAuthority: sharedAuthority,
  });

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: sharedAuthority,
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
    assets: [asset.publicKey],
  });
});

test('it allows collection update authority to remove collection-managed assets from a group', async (t) => {
  const umi = await createUmi();
  const sharedAuthority = generateSigner(umi);

  const group = await createGroup(umi, { updateAuthority: sharedAuthority });
  const { asset, collection } = await createAssetWithCollection(umi, {
    updateAuthority: sharedAuthority,
  });

  await addAssetsToGroup(umi, {
    group: group.publicKey,
    authority: sharedAuthority,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
      { isSigner: false, isWritable: false, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await removeAssetsFromGroup(umi, {
    group: group.publicKey,
    authority: sharedAuthority,
    assets: [asset.publicKey],
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: asset.publicKey },
      { isSigner: false, isWritable: false, pubkey: collection.publicKey },
    ])
    .sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    updateAuthority: sharedAuthority.publicKey,
    assets: [],
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

test.serial(
  'it rejects addAssetsToGroup when the group asset vector is already at max size',
  async (t) => {
    const umi = await createUmi();
    const group = await createGroup(umi);

    const maxGroupVectorSize = 256;
    const batchSize = 8;
    let batch: PublicKey[] = [];

    for (let i = 0; i < maxGroupVectorSize; i += 1) {
      const asset = await createAsset(umi);
      batch.push(asset.publicKey);

      if (batch.length === batchSize) {
        await addAssetsToGroup(umi, {
          group: group.publicKey,
          authority: umi.identity,
        })
          .addRemainingAccounts(
            batch.map((pubkey) => ({
              isSigner: false,
              isWritable: true,
              pubkey,
            }))
          )
          .sendAndConfirm(umi);

        batch = [];
      }
    }

    if (batch.length > 0) {
      await addAssetsToGroup(umi, {
        group: group.publicKey,
        authority: umi.identity,
      })
        .addRemainingAccounts(
          batch.map((pubkey) => ({
            isSigner: false,
            isWritable: true,
            pubkey,
          }))
        )
        .sendAndConfirm(umi);
    }

    const overflowAsset = await createAsset(umi);
    const error = await t.throwsAsync(
      addAssetsToGroup(umi, {
        group: group.publicKey,
        authority: umi.identity,
      })
        .addRemainingAccounts([
          {
            isSigner: false,
            isWritable: true,
            pubkey: overflowAsset.publicKey,
          },
        ])
        .sendAndConfirm(umi)
    );

    t.truthy(error);
    t.regex((error as Error).message, /Group vector is at maximum capacity/);
  }
);

test.serial(
  'it rejects addCollectionsToGroup when the group collection vector is already at max size',
  async (t) => {
    const umi = await createUmi();
    const group = await createGroup(umi);

    const maxGroupVectorSize = 256;
    const batchSize = 8;
    let batch: PublicKey[] = [];

    for (let i = 0; i < maxGroupVectorSize; i += 1) {
      const collection = await createCollection(umi);
      batch.push(collection.publicKey);

      if (batch.length === batchSize) {
        await addCollectionsToGroup(umi, {
          group: group.publicKey,
          authority: umi.identity,
        })
          .addRemainingAccounts(
            batch.map((pubkey) => ({
              isSigner: false,
              isWritable: true,
              pubkey,
            }))
          )
          .sendAndConfirm(umi);

        batch = [];
      }
    }

    if (batch.length > 0) {
      await addCollectionsToGroup(umi, {
        group: group.publicKey,
        authority: umi.identity,
      })
        .addRemainingAccounts(
          batch.map((pubkey) => ({
            isSigner: false,
            isWritable: true,
            pubkey,
          }))
        )
        .sendAndConfirm(umi);
    }

    const overflowCollection = await createCollection(umi);
    const error = await t.throwsAsync(
      addCollectionsToGroup(umi, {
        group: group.publicKey,
        authority: umi.identity,
      })
        .addRemainingAccounts([
          {
            isSigner: false,
            isWritable: true,
            pubkey: overflowCollection.publicKey,
          },
        ])
        .sendAndConfirm(umi)
    );

    t.truthy(error);
    t.regex((error as Error).message, /Group vector is at maximum capacity/);
  }
);
