import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { NonFungible, create, fetchNonFungible } from '../src';
import { createUmi } from './_setup';

test('it can create new accounts', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const nonFungible = await fetchNonFungible(umi, assetAddress.publicKey);
  console.log(nonFungible);
  t.like(nonFungible, <NonFungible>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});
