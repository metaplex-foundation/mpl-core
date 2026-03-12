import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { addGroupsToGroup, removeGroupsFromGroup } from '../src';
import { assertGroup, createGroup, createUmi } from './_setupRaw';

// -----------------------------------------------------------------------------
// Complex Group Relations – Parent ↔ Child Synchronisation
// -----------------------------------------------------------------------------

test('it keeps parentGroups in sync with groups when adding and removing child groups', async (t) => {
  // ---------------------------------------------------------------------------
  // 1. Setup – create a parent and a child group.
  // ---------------------------------------------------------------------------
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });
  const child = await createGroup(umi, { name: 'child' });

  // ---------------------------------------------------------------------------
  // 2. Add the child to the parent.
  // ---------------------------------------------------------------------------
  await addGroupsToGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child.publicKey },
    ])
    .sendAndConfirm(umi);

  // Parent should list child, and child should list parent.
  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [child.publicKey],
  });

  await assertGroup(t, umi, {
    group: child.publicKey,
    updateAuthority: umi.identity.publicKey,
    parentGroups: [parent.publicKey],
  });

  // ---------------------------------------------------------------------------
  // 3. Remove the child again.
  // ---------------------------------------------------------------------------
  await removeGroupsFromGroup(umi, {
    parentGroup: parent.publicKey,
    groups: [child.publicKey],
    authority: umi.identity,
  })
    .addRemainingAccounts([
      { isSigner: false, isWritable: true, pubkey: child.publicKey },
    ])
    .sendAndConfirm(umi);

  // Relations should now be cleared on both sides.
  await assertGroup(t, umi, {
    group: parent.publicKey,
    updateAuthority: umi.identity.publicKey,
    groups: [],
  });

  await assertGroup(t, umi, {
    group: child.publicKey,
    updateAuthority: umi.identity.publicKey,
    parentGroups: [],
  });
});

test('it rejects adding a parent group as its own child group', async (t) => {
  const umi = await createUmi();
  const parent = await createGroup(umi, { name: 'parent' });

  await t.throwsAsync(
    addGroupsToGroup(umi, {
      parentGroup: parent.publicKey,
      groups: [parent.publicKey],
      authority: umi.identity,
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: parent.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'IncorrectAccount' }
  );
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
