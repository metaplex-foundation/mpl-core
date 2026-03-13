import test from 'ava';
import { generateSigner, sol } from '@metaplex-foundation/umi';
import { SPL_SYSTEM_PROGRAM_ID } from '@metaplex-foundation/mpl-toolbox';
import {
  pluginAuthorityPair,
  addressPluginAuthority,
  updatePluginAuthority,
  burnV1,
  transferV1,
  ruleSet,
} from '../../../src';
import {
  assertAsset,
  assertBurned,
  createAsset,
  createAssetWithCollection,
  createUmi,
} from '../../_setupRaw';

// ---------------------------------------------------------------------------
// Gap 1: PermanentBurnDelegate burning a frozen asset (ForceApproved overrides
// freeze rejection on burn)
// ---------------------------------------------------------------------------

test('it can burn a frozen asset using PermanentBurnDelegate (ForceApproved overrides freeze)', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(10));
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: true,
    },
    permanentBurnDelegate: {
      authority: { type: 'Address', address: delegate.publicKey },
    },
  });

  await burnV1(umi, {
    asset: asset.publicKey,
    authority: delegate,
    payer: owner,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('it can burn a frozen asset using collection PermanentBurnDelegate (ForceApproved overrides freeze)', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(10));

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner: owner.publicKey,
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentBurnDelegate',
          authority: updatePluginAuthority(),
        }),
      ],
    }
  );

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });

  await burnV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    authority: owner,
    payer: owner,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

// ---------------------------------------------------------------------------
// Gap 2: PermanentTransferDelegate overriding Royalties rejection
// ---------------------------------------------------------------------------

test('it can transfer with PermanentTransferDelegate even when collection Royalties would reject', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(10));
  const delegate = generateSigner(umi);

  // Create a program-owned account to transfer to (triggers royalty check).
  const programOwned = await createAsset(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner: owner.publicKey,
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    },
    {
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 500,
            creators: [{ address: owner.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
          },
        }),
      ],
    }
  );

  // Transfer to a program-owned address NOT on the allow list.
  // Royalties would normally reject, but PermanentTransferDelegate ForceApproves.
  await transferV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    authority: delegate,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwned.publicKey,
  });
});

// ---------------------------------------------------------------------------
// Gap 3: Asset-level PermanentFreezeDelegate overrides collection-level
// PermanentFreezeDelegate (same PluginType key in BTreeMap, asset wins)
// ---------------------------------------------------------------------------

test('asset PermanentFreezeDelegate(unfrozen) overrides collection PermanentFreezeDelegate(frozen) for transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner: owner.publicKey,
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
        }),
      ],
    },
    {
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    }
  );

  // Asset's unfrozen PermanentFreezeDelegate should override collection's frozen one.
  await transferV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    authority: owner,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('asset PermanentFreezeDelegate(frozen) blocks transfer even when collection PermanentFreezeDelegate is unfrozen', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  await umi.rpc.airdrop(owner.publicKey, sol(10));
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner: owner.publicKey,
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      payer: owner,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
        }),
      ],
    }
  );

  // Asset's frozen PermanentFreezeDelegate should block transfer regardless of collection.
  const result = transferV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    authority: owner,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});
