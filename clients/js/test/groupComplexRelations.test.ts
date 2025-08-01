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
