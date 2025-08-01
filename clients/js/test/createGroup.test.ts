import test from 'ava';
import {
  assertGroup,
  createGroup,
  createUmi,
  DEFAULT_GROUP,
} from './_setupRaw';

// Verify creation of a group

test('it can create a new group', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi, {
    name: 'My Group',
  });

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    name: 'My Group',
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});
