import test from 'ava';
import { pluginAuthorityPair, updateV1 } from '../../../src';
import { createAsset, createUmi } from '../../_setup';

test('it can prevent the asset from metadata updating', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'ImmutableMetadata',
      }),
    ],
  });

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'bread',
    newUri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
