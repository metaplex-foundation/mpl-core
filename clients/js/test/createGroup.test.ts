import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { createGroupV1, RelationshipKind } from '../src';
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

test('it rejects creating a group with itself as a child relationship', async (t) => {
  const umi = await createUmi();
  const group = generateSigner(umi);

  await t.throwsAsync(
    createGroupV1(umi, {
      name: 'Self Child Group',
      uri: DEFAULT_GROUP.uri,
      group,
      payer: umi.identity,
      relationships: [
        { kind: RelationshipKind.ChildGroup, key: group.publicKey },
      ],
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: group.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'IncorrectAccount' }
  );
});

test('it rejects creating a group with itself as a parent relationship', async (t) => {
  const umi = await createUmi();
  const group = generateSigner(umi);

  await t.throwsAsync(
    createGroupV1(umi, {
      name: 'Self Parent Group',
      uri: DEFAULT_GROUP.uri,
      group,
      payer: umi.identity,
      relationships: [
        { kind: RelationshipKind.ParentGroup, key: group.publicKey },
      ],
    })
      .addRemainingAccounts([
        { isSigner: false, isWritable: true, pubkey: group.publicKey },
      ])
      .sendAndConfirm(umi),
    { name: 'IncorrectAccount' }
  );
});
