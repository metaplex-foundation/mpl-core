import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approvePluginAuthority,
  transfer,
  updateAuthority,
  plugin,
  authority,
} from '../../../src';
import { DEFAULT_ASSET, assertAsset, createAsset, createUmi } from '../../_setup';

test('a delegate can transfer the asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [plugin('Transfer', [{}])],
  })

  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Transfer,
    newAuthority: authority('Pubkey', { address: delegateAddress.publicKey }),
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwnerAddress.publicKey,
    authority: delegateAddress,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwnerAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    plugins: [{
      authority: authority('Pubkey', { address: delegateAddress.publicKey }),
      plugin: plugin('Transfer', [{}])
    }],
  });
});
