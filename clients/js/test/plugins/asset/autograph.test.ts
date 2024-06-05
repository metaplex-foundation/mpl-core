import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { DEFAULT_ASSET, assertAsset, createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';
import { addPlugin, updatePlugin } from '../../../src';

test('it can create asset with autograph plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Autograph',
        signatures: [],
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [],
    },
  });
});

test('it cannot create asset with autograph plugin and unauthorized signature', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const autograph = generateSigner(umi);

  const res = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: autograph.publicKey,
            message: 'hi',
          },
        ],
      },
    ],
  });

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it can create asset with autograph plugin with authorized signature', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'hi',
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
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'hi',
        },
      ],
    },
  });
});

test('it cannot add autograph plugin to asset by creator', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  const res = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator',
        },
      ],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it can add autograph plugin to asset by owner', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: owner.publicKey,
          message: 'owner',
        },
      ],
    },
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: owner.publicKey,
          message: 'owner',
        },
      ],
    },
  });
});

test('it cannot add autograph plugin to asset by 3rd party', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const autograph = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  const res = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: autograph.publicKey,
          message: 'autograph',
        },
      ],
    },
    authority: autograph,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'NoApprovals' });
});

test('it cannot add autograph to asset by unauthorized 3rd party', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const unauthed = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator',
        },
        {
          address: owner.publicKey,
          message: 'owner',
        },
      ],
    },
    authority: unauthed,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it can add additional autograph to asset via update by 3rd party', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const autograph = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator',
        },
        {
          address: autograph.publicKey,
          message: 'autograph',
        },
      ],
    },
    authority: autograph,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator',
        },
        {
          address: autograph.publicKey,
          message: 'autograph',
        },
      ],
    },
  });
});

test('it can remove autograph if owner', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [],
    },
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [],
    },
  });
});

test('it cannot remove autograph if not owner', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const autograph = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [],
    },
    authority: autograph,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'MissingSigner' });
});

test('it cannot modify autograph message as signer', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator2',
        },
      ],
    },
    authority: umi.identity,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidPluginOperation' });
});

test('it cannot modify autograph message as owner', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator',
        },
        {
          address: owner.publicKey,
          message: 'owner',
        },
      ],
    },
    authority: owner,
  }).sendAndConfirm(umi);

  const res = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: umi.identity.publicKey,
          message: 'creator2',
        },
        {
          address: owner.publicKey,
          message: 'owner2',
        },
      ],
    },
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(res, { name: 'InvalidPluginOperation' });
});

test('it can remove and add autographs as owner', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: owner.publicKey,
          message: 'owner',
        },
      ],
    },
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: owner.publicKey,
          message: 'owner',
        },
      ],
    },
  });
});

test('it can remove and add autographs as delegate', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const delegate = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
        ],
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Autograph',
      signatures: [
        {
          address: delegate.publicKey,
          message: 'delegate',
        },
      ],
    },
    authority: delegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    autograph: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      signatures: [
        {
          address: delegate.publicKey,
          message: 'delegate',
        },
      ],
    },
  });
});

test('it cannot add duplicate autographs', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);

  const res = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Autograph',
        signatures: [
          {
            address: umi.identity.publicKey,
            message: 'creator',
          },
          {
            address: umi.identity.publicKey,
            message: 'creator2',
          },
        ],
      },
    ],
  });

  await t.throwsAsync(res, { name: 'InvalidPluginSetting' });
});
