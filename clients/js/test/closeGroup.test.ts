import test from 'ava';
import { closeGroup } from '../src';
import { assertBurned, createGroup, createUmi } from './_setupRaw';

test('it can close a group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  await closeGroup(umi, {
    group: group.publicKey,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, group.publicKey);
});
