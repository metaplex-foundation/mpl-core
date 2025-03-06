import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { assertAsset, createAsset, createUmi } from '../_setupRaw';
import {
  addressPluginAuthority,
  pluginAuthorityPair,
  legacyDelegate,
} from '../../src';
import { ERR_CANNOT_DELEGATE } from '../../src/instructions/errors';

test('it can delegate with a new delegate address and no defined plugins', async (t) => {
  // Given an Umi instance, asset with no plugins and new delegate.
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const delegate = generateSigner(umi);

  // It adds all required plugins and sets the new delegate as their authority.
  await legacyDelegate(umi, asset, delegate.publicKey).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
  });
});

test('it can delegate with a new delegate address and one defined plugin', async (t) => {
  // Given an Umi instance, asset with one defined plugin and new delegate.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });
  const delegate = generateSigner(umi);

  // It adds all missing plugins and sets the new delegate as the authority of all required plugins.
  await legacyDelegate(umi, asset, delegate.publicKey).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
  });
});

test('it can delegate with a new delegate address and all required plugins defined', async (t) => {
  // Given an Umi instance, asset with all required plugins and new delegate.
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
  const delegate = generateSigner(umi);

  // It sets the new delegate as the authority of all defined plugins.
  await legacyDelegate(umi, asset, delegate.publicKey).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
  });
});

test('it cannot delegate if the target delegate is the asset owner', async (t) => {
  // Given an Umi instance and an asset with the owner plugin authority.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  // The delegate error is expected.
  t.throws(() => legacyDelegate(umi, asset, asset.owner), {
    message: ERR_CANNOT_DELEGATE,
  });
});

test('it cannot delegate if the target delegate is the asset owner and the plugin authority is an explicit public key of the asset owner', async (t) => {
  // Given an Umi instance and an asset with an explicit public key of the asset owner as the plugin authority.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(umi.identity.publicKey),
      }),
    ],
  });

  // The delegate error is expected.
  t.throws(() => legacyDelegate(umi, asset, asset.owner), {
    message: ERR_CANNOT_DELEGATE,
  });
});

test('it cannot delegate if the target delegate is already set as an authority of a plugin', async (t) => {
  // Given an Umi instance and an asset with the target delegate as the initial plugin authority.
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  // The delegate error is expected.
  t.throws(() => legacyDelegate(umi, asset, delegate.publicKey), {
    message: ERR_CANNOT_DELEGATE,
  });
});
