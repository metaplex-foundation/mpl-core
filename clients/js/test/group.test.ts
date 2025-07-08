import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { getGroupV1GpaBuilder, Key } from '../src';
import { createGroup, createUmi } from './_setupRaw';

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
