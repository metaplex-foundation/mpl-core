import { generateSigner, sol } from '@metaplex-foundation/umi';
import test from 'ava';

import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { burnV1, pluginAuthorityPair } from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertBurned,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from './_setupRaw';

test('it can burn an asset as the owner', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterAsset = await assertBurned(t, umi, asset.publicKey);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
});

test('it cannot burn an asset if not the owner', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot burn an asset as the authority', async (t) => {
  const umi = await createUmi();
  const authority = generateSigner(umi);

  const asset = await createAsset(umi, { updateAuthority: authority });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: authority.publicKey },
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
    authority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: authority.publicKey },
  });
});

test('it cannot burn an asset if it is frozen', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it cannot burn asset in collection if no collection specified', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(umi, {});

  const result = burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it cannot burn an asset if collection permanently frozen', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    permanentFreezeDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it cannot use an invalid system program for assets', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const fakeSystemProgram = generateSigner(umi);
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);
  const fakeLogWrapper = generateSigner(umi);
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it can burn using owner authority', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const asset = await createAsset(umi, {
    owner: owner.publicKey,
  });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await burnV1(umi, {
    asset: asset.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterAsset = await assertBurned(t, umi, asset.publicKey);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
});

test('it cannot burn an asset with the wrong collection specified', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);
  const wrongCollection = await createCollection(umi);

  const result = burnV1(umi, {
    asset: asset.publicKey,
    collection: wrongCollection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('it can burn asset with different payer', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const asset = await createAsset(umi, {
    owner: owner.publicKey,
  });
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const lamportsBefore = await umi.rpc.getBalance(umi.identity.publicKey);

  await burnV1(umi, {
    asset: asset.publicKey,
    payer: umi.identity,
    authority: owner,
  }).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterAsset = await assertBurned(t, umi, asset.publicKey);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));

  const lamportsAfter = await umi.rpc.getBalance(umi.identity.publicKey);

  t.true(lamportsAfter.basisPoints > lamportsBefore.basisPoints);
});
