import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { MyAccount, create, fetchMyAccount } from '../src';
import { createUmi } from './_setup';

test('it can create new accounts', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const address = generateSigner(umi);

  // When we create a new account.
  await create(umi, { address, arg1: 1, arg2: 2 }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  t.like(await fetchMyAccount(umi, address.publicKey), <MyAccount>{
    publicKey: address.publicKey,
    authority: umi.identity.publicKey,
    data: { field1: 1, field2: 2 },
  });
});
