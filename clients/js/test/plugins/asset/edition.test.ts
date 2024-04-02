import test from 'ava';
import { publicKey } from '@metaplex-foundation/umi';
import {
  fetchAssetV1,
  // pluginAuthorityPair,

} from '../../../src';
import {
  // DEFAULT_ASSET,
  // assertAsset,
  // createAsset,
  createUmi,
} from '../../_setup';

test('it can create asset with edition plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Edition',
        data: { number: 1 },
      })
    ],
  });

  console.log(asset.publicKey)

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    edition: {
      authority: {
        type: 'UpdateAuthority',
      },
      number: BigInt(1)
    },
  });
})
