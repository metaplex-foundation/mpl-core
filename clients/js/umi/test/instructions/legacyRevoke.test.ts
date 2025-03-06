import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { assertAsset, createAsset, createUmi } from '../_setupRaw';
import {
  pluginAuthorityPair,
  addressPluginAuthority,
  legacyRevoke,
} from '../../src';
import { ERR_CANNOT_REVOKE } from '../../src/instructions/errors';

test('it can revoke with one plugin defined', async (t) => {
  // Given an Umi instance and asset with one required plugin with delegated authority.
  const umi = await createUmi();
  const delegateToRevokeFrom = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  await legacyRevoke(umi, asset).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke with all required plugins defined', async (t) => {
  // Given an Umi instance and asset with all required plugins with delegated authorities.
  const umi = await createUmi();
  const delegateToRevokeFrom = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  await legacyRevoke(umi, asset).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke with a couple of non-homogenous plugin authorities', async (t) => {
  // Given an Umi instance and asset with a couple required plugins with non-homogenous plugin authorities.
  const umi = await createUmi();
  const delegateToRevokeFrom = generateSigner(umi);
  const delegateToRevokeFrom2 = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom2.publicKey),
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  await legacyRevoke(umi, asset).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke with all required plugins with non-homogenous plugin authorities', async (t) => {
  // Given an Umi instance and asset with all required plugin with non-homogenous plugin authorities.
  const umi = await createUmi();
  const delegateToRevokeFrom = generateSigner(umi);
  const delegateToRevokeFrom2 = generateSigner(umi);
  const delegateToRevokeFrom3 = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
        data: {
          frozen: false,
        },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom2.publicKey),
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom3.publicKey),
      }),
    ],
  });

  // The authority of all defined plugins becomes the asset owner.
  await legacyRevoke(umi, asset).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot revoke if no plugins defined', async (t) => {
  // Given an Umi instance and asset with no plugins defined.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  // The revoke error is expected.
  t.throws(() => legacyRevoke(umi, asset), {
    message: ERR_CANNOT_REVOKE,
  });
});

test('it cannot revoke if one of plugin authorities is the asset owner', async (t) => {
  // Given an Umi instance and asset with all required plugins defined, and all the plugins except one have a delegated authority.
  const umi = await createUmi();
  const delegateToRevokeFrom = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: addressPluginAuthority(delegateToRevokeFrom.publicKey),
      }),
    ],
  });

  // The revoke error is expected.
  t.throws(() => legacyRevoke(umi, asset), {
    message: ERR_CANNOT_REVOKE,
  });
});

test('it cannot revoke if all of the plugin authorities are the asset owner', async (t) => {
  // Given an Umi instance and asset with all required plugins defined with the asset owner as the plugin authority.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  // The revoke error is expected.
  t.throws(() => legacyRevoke(umi, asset), {
    message: ERR_CANNOT_REVOKE,
  });
});
