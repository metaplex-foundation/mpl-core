import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { updateGroup } from '../src';
import {
  assertGroup,
  createGroup,
  createUmi,
  DEFAULT_GROUP,
} from './_setupRaw';

// -----------------------------------------------------------------------------
// Update Authority Transfer
// -----------------------------------------------------------------------------

test("it can transfer a group's update authority", async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  const newAuthority = generateSigner(umi);

  // 1. Transfer the update authority to the new signer.
  await updateGroup(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    newUpdateAuthority: newAuthority.publicKey,
    newName: null,
    newUri: null,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: newAuthority.publicKey,
  });

  // 2. The new authority updates the group's name.
  const UPDATED_NAME = 'Updated Group Name';

  await updateGroup(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: newAuthority,
    newName: UPDATED_NAME,
    newUri: null,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    group: group.publicKey,
    name: UPDATED_NAME,
    updateAuthority: newAuthority.publicKey,
  });

  // 3. Old authority attempting further updates should fail.
  await t.throwsAsync(
    updateGroup(umi, {
      group: group.publicKey,
      payer: umi.identity,
      authority: umi.identity,
      newName: 'Should Fail',
      newUri: null,
    }).sendAndConfirm(umi)
  );
});
