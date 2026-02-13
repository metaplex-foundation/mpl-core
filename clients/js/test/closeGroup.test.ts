import test from 'ava';
import { addAssetsToGroup, closeGroup } from '../src';
import { assertBurned, createAsset, createGroup, createUmi } from './_setupRaw';

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
