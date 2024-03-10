import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approvePluginAuthority,
  addPlugin,
  updateAuthority,
  plugin,
  getPubkeyAuthority,
} from '../src';
import { DEFAULT_ASSET, assertAsset, createAsset, createUmi } from './_setup';

test('it can add an authority to a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {})

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  })

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  })
    .append(
      approvePluginAuthority(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.Freeze,
        newAuthority: getPubkeyAuthority(delegateAddress.publicKey),
      })
    )
    .sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        pubkey: [delegateAddress.publicKey],
      },
      frozen: false,
    },
  });
});
