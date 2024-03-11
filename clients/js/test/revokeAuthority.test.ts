import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  PluginType,
  approvePluginAuthority,
  plugin,
  revokePluginAuthority,
  updateAuthority,
  getPubkeyAuthority,
  getNoneAuthority,
} from '../src';
import { assertAsset, createAsset, createUmi } from './_setup';

test('it can remove an authority from a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      { plugin: plugin('Freeze', [{ frozen: false }]), authority: null },
    ],
  });

  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: getPubkeyAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Freeze,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can remove the default authority from a plugin to make it immutable', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      { plugin: plugin('Freeze', [{ frozen: false }]), authority: null },
    ],
  });

  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: getNoneAuthority(),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'None',
      },
      frozen: false,
    },
  });
});

test('it can remove a pubkey authority from a plugin if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const pubkeyAuth = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    plugins: [
      { plugin: plugin('Freeze', [{ frozen: false }]), authority: null },
    ],
  });

  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Freeze,
    newAuthority: getPubkeyAuthority(pubkeyAuth.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Pubkey',
        address: pubkeyAuth.publicKey,
      },
      frozen: false,
    },
  });

  const umi2 = await createUmi();

  await revokePluginAuthority(umi2, {
    payer: umi2.identity,
    asset: asset.publicKey,
    authority: pubkeyAuth,
    pluginType: PluginType.Freeze,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it cannot remove a none authority from a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      { plugin: plugin('Freeze', [{ frozen: false }]), authority: null },
    ],
  });

  await approvePluginAuthority(umi, {
    payer: umi.identity,
    asset: asset.publicKey,
    authority: umi.identity,
    pluginType: PluginType.Freeze,
    newAuthority: getNoneAuthority(),
  }).sendAndConfirm(umi);

  const err = await t.throwsAsync(() =>
    revokePluginAuthority(umi, {
      payer: umi.identity,
      asset: asset.publicKey,
      authority: umi.identity,
      pluginType: PluginType.Freeze,
    }).sendAndConfirm(umi)
  );

  t.true(err?.message.startsWith('Invalid Authority'));
});
