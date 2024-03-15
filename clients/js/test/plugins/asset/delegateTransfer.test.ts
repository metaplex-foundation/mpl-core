import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  PluginType,
  approvePluginAuthority,
  transfer,
  authority,
  pluginAuthorityPair,
  revokePluginAuthority,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('a delegate can transfer the asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [pluginAuthorityPair({ type: 'Transfer' })],
  });

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
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transfer: {
      authority: {
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
    },
  });
});

test('owner can transfer asset with delegate transfer', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Transfer',
        authority: authority('Pubkey', { address: delegateAddress.publicKey }),
      }),
    ],
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwnerAddress.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwnerAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transfer: {
      authority: {
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
    },
  });
});

test('it can revoke a delegate transfer plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Transfer',
        authority: authority('Pubkey', { address: delegateAddress.publicKey }),
      }),
    ],
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Transfer,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transfer: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot transfer after delegate has been revoked', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Transfer',
        authority: authority('Pubkey', { address: delegateAddress.publicKey }),
      }),
    ],
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Transfer,
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwnerAddress.publicKey,
    authority: delegateAddress,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transfer: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
