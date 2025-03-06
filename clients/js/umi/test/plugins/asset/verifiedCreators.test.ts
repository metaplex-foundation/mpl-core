import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { DEFAULT_ASSET, assertAsset, createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';
import { addPlugin, updatePlugin } from '../../../src';

test('it can create asset with verified creators plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [],
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [],
    },
  });
});

test('it cannot create asset with verified creators plugin and unauthorized signature', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);

  const res = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: true,
          },
        ],
      },
    ],
  });

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it can create asset with verified creators plugin with authorized signature', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: umi.identity.publicKey,
            verified: true,
          },
        ],
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it cannot add verified creator plugin to asset by owner', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  const res = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: owner.publicKey,
          verified: true,
        },
      ],
    },
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it can create asset with verified creators plugin with unverified signatures and then verify', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: umi.identity.publicKey,
            verified: false,
          },
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: false,
        },
        {
          address: creator.publicKey,
          verified: false,
        },
      ],
    },
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: false,
        },
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it can unverify signature verified creator plugin', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: false,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.publicKey,
          verified: false,
        },
      ],
    },
  });
});

test('it cannot verify a verified creator plugin with unauthorized signature', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);
  const unauthed = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: unauthed,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it cannot remove verified creator plugin signture with unauthorized signature', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);
  const unauthed = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [],
    },
    authority: unauthed,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it can remove and add unverified creator plugin signature with update auth', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);
  const creator2 = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
        {
          address: creator2.publicKey,
          verified: false,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          verified: true,
        },
      ],
    },
  });
});

test('it cannot remove verified creator plugin signature with update auth', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidPluginOperation' });
});

test('it cannot unverify verified creator plugin signature with update auth', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const creator = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: creator.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
    authority: creator,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.publicKey,
          verified: true,
        },
      ],
    },
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'VerifiedCreators',
      signatures: [
        {
          address: creator.publicKey,
          verified: false,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidPluginOperation' });
});

test('it cannot add duplicate verified creator signatures', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const res = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'VerifiedCreators',
        signatures: [
          {
            address: umi.identity.publicKey,
            verified: false,
          },
          {
            address: umi.identity.publicKey,
            verified: false,
          },
        ],
      },
    ],
  });

  await t.throwsAsync(res, { name: 'InvalidPluginSetting' });
});
