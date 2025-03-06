import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  PluginType,
  approvePluginAuthorityV1,
  revokePluginAuthorityV1,
  addressPluginAuthority,
  nonePluginAuthority,
  pluginAuthorityPair,
  approveCollectionPluginAuthorityV1,
  revokeCollectionPluginAuthorityV1,
  updatePluginAuthority,
  ownerPluginAuthority,
} from '../src';
import {
  assertAsset,
  assertCollection,
  createAsset,
  createCollection,
  createUmi,
} from './_setupRaw';

test('it can remove an authority from a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  await revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

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
  });
});

test('it can remove the default authority from a plugin to make it immutable', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: nonePluginAuthority(),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'None',
      },
      frozen: false,
    },
  });
});

test('it can remove a pubkey authority from an owner-managed plugin if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const pubkeyAuth = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(pubkeyAuth.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: pubkeyAuth.publicKey,
      },
      frozen: false,
    },
  });

  const umi2 = await createUmi();

  await revokePluginAuthorityV1(umi2, {
    payer: umi2.identity,
    asset: asset.publicKey,
    authority: pubkeyAuth,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

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
  });
});

test('it can remove an update authority from an owner-managed plugin if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    authority: owner,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: updatePluginAuthority(),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: { type: 'UpdateAuthority' },
      frozen: false,
    },
  });

  await revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can remove a pubkey authority from an authority-managed plugin if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const pubkeyAuth = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Attributes,
    newAuthority: addressPluginAuthority(pubkeyAuth.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'Address',
        address: pubkeyAuth.publicKey,
      },
      attributeList: [],
    },
  });

  const umi2 = await createUmi();

  await revokePluginAuthorityV1(umi2, {
    payer: umi2.identity,
    asset: asset.publicKey,
    authority: pubkeyAuth,
    pluginType: PluginType.Attributes,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });
});

test('it can remove an owner authority from an authority-managed plugin if that pubkey is the signer authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    plugins: [
      pluginAuthorityPair({ type: 'Attributes', data: { attributeList: [] } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Attributes,
    newAuthority: ownerPluginAuthority(),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'Owner',
      },
      attributeList: [],
    },
  });

  const umi2 = await createUmi();

  await revokePluginAuthorityV1(umi2, {
    payer: umi2.identity,
    asset: asset.publicKey,
    authority: owner,
    pluginType: PluginType.Attributes,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
  });
});

test('it cannot remove a none authority from a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    payer: umi.identity,
    asset: asset.publicKey,
    authority: umi.identity,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: nonePluginAuthority(),
  }).sendAndConfirm(umi);

  const err = await t.throwsAsync(() =>
    revokePluginAuthorityV1(umi, {
      payer: umi.identity,
      asset: asset.publicKey,
      authority: umi.identity,
      pluginType: PluginType.FreezeDelegate,
    }).sendAndConfirm(umi)
  );

  t.true(err?.message.startsWith('Invalid Authority'));
});

test('it cannot use an invalid system program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const fakeSystemProgram = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  const result = revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const fakeLogWrapper = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  const result = revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot use an invalid system program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const fakeSystemProgram = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = revokeCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const fakeLogWrapper = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await approveCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = revokeCollectionPluginAuthorityV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it can revoke an authority from a plugin if another plugin is None', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
      pluginAuthorityPair({
        authority: nonePluginAuthority(),
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await approvePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    newAuthority: addressPluginAuthority(delegateAddress.publicKey),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });

  await revokePluginAuthorityV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

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
  });
});
