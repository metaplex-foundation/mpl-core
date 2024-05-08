import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';

import { addPlugin } from '../../../src';
import { DEFAULT_ASSET, assertAsset, createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';

test('it cannot add UA-managed plugin if addBlocker had been added on creation', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AddBlocker',
      },
    ],
  });

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add plugins unless AddBlocker is added', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Attributes',

      attributeList: [],
    },
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'AddBlocker',
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [],
    },
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Attributes',

      attributeList: [],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add owner-managed plugins even if AddBlocker had been added', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AddBlocker',
      },
    ],
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: { type: 'FreezeDelegate', frozen: false },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it states that UA is the only one who can add the AddBlocker', async (t) => {
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);
  const randomUser = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: updateAuthority.publicKey,
  });

  // random keypair can't add AddBlocker
  let result = addPlugin(umi, {
    authority: randomUser,
    asset: asset.publicKey,
    plugin: {
      type: 'AddBlocker',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'NoApprovals',
  });

  // Owner can't add AddBlocker
  result = addPlugin(umi, {
    authority: umi.identity,
    asset: asset.publicKey,
    plugin: {
      type: 'AddBlocker',
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'NoApprovals',
  });

  // UA CAN add AddBlocker
  await addPlugin(umi, {
    authority: updateAuthority,
    asset: asset.publicKey,
    plugin: { type: 'AddBlocker' },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    addBlocker: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});
