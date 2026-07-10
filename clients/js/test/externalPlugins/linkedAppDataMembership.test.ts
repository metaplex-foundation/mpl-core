/* eslint-disable no-restricted-syntax */
/* eslint-disable no-await-in-loop */
import test from 'ava';
import { publicKey } from '@metaplex-foundation/umi';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { createUmi } from '../_setupRaw';
import { createAsset, createCollection } from '../_setupSdk';
import {
  ExternalPluginAdapterSchema,
  fetchAssetV1,
  writeData,
} from '../../src';

/*
 * `WriteExternalPluginAdapterDataV1` resolves linked external plugin adapters
 * (e.g. `LinkedAppData`) from the supplied collection account. These tests
 * verify that the write is only honored when the asset is actually a member
 * of that collection, i.e.
 *   asset.update_authority == UpdateAuthority::Collection(collection.key)
 * Writes referencing a collection the asset does not belong to must fail with
 * `InvalidCollection`.
 */

test('LinkedAppData write is rejected when the asset is not a member of the supplied collection', async (t) => {
  const umi = await createUmi();

  // A collection whose LinkedAppData names its creator as the explicit
  // Address(...) data authority.
  const collectionAuthority = await generateSignerWithSol(umi);
  const dataAuthority = {
    type: 'Address' as const,
    address: collectionAuthority.publicKey,
  };

  const collection = await createCollection(umi, {
    payer: collectionAuthority,
    updateAuthority: collectionAuthority,
    plugins: [
      {
        type: 'LinkedAppData',
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  // A standalone asset owned by an unrelated wallet – not a member of any
  // collection.
  const assetOwner = await generateSignerWithSol(umi);
  const standaloneAsset = await createAsset(umi, {
    payer: assetOwner,
    owner: assetOwner.publicKey,
    updateAuthority: assetOwner.publicKey,
  });

  t.deepEqual(
    standaloneAsset.updateAuthority,
    { type: 'Address', address: assetOwner.publicKey },
    'asset is not a member of any collection'
  );

  const payload = new TextEncoder().encode('some data');

  await t.throwsAsync(
    writeData(umi, {
      key: {
        type: 'LinkedAppData',
        dataAuthority,
      },
      payer: collectionAuthority,
      authority: collectionAuthority,
      collection: collection.publicKey,
      asset: standaloneAsset.publicKey,
      data: payload,
    }).sendAndConfirm(umi),
    { name: 'InvalidCollection' },
    'LinkedAppData write against a non-member asset must be rejected'
  );

  // Verify the asset was untouched.
  const fetched = await fetchAssetV1(umi, publicKey(standaloneAsset.publicKey));

  t.deepEqual(
    fetched.updateAuthority,
    { type: 'Address', address: assetOwner.publicKey },
    'asset update authority must be unchanged'
  );

  const section = (fetched.dataSections ?? []).find(
    (s) =>
      s.parentKey.type === 'LinkedAppData' &&
      s.parentKey.dataAuthority.type === 'Address' &&
      s.parentKey.dataAuthority.address === collectionAuthority.publicKey
  );
  t.is(
    section,
    undefined,
    'no DataSection should be present on the non-member asset'
  );
});

test('LinkedAppData writes from multiple non-member collections are all rejected', async (t) => {
  const umi = await createUmi();

  // A standalone asset that belongs to no collection.
  const assetOwner = await generateSignerWithSol(umi);
  const standaloneAsset = await createAsset(umi, {
    payer: assetOwner,
    owner: assetOwner.publicKey,
    updateAuthority: assetOwner.publicKey,
  });

  for (let i = 0; i < 2; i += 1) {
    const collectionAuthority = await generateSignerWithSol(umi);
    const dataAuthority = {
      type: 'Address' as const,
      address: collectionAuthority.publicKey,
    };

    const collection = await createCollection(umi, {
      payer: collectionAuthority,
      updateAuthority: collectionAuthority,
      plugins: [
        {
          type: 'LinkedAppData',
          dataAuthority,
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    });

    await t.throwsAsync(
      writeData(umi, {
        key: { type: 'LinkedAppData', dataAuthority },
        payer: collectionAuthority,
        authority: collectionAuthority,
        collection: collection.publicKey,
        asset: standaloneAsset.publicKey,
        data: Uint8Array.from([0xab, 0xcd, i, i, i, i]),
      }).sendAndConfirm(umi),
      { name: 'InvalidCollection' },
      `non-member write #${i} must be rejected`
    );
  }

  const fetched = await fetchAssetV1(umi, publicKey(standaloneAsset.publicKey));
  t.is(
    (fetched.dataSections ?? []).length,
    0,
    'asset must have no DataSections'
  );
});

test('LinkedAppData write succeeds for a legitimate collection member', async (t) => {
  const umi = await createUmi();

  const signer = await generateSignerWithSol(umi);
  const dataAuthority = {
    type: 'Address' as const,
    address: signer.publicKey,
  };

  // Collection with LinkedAppData(Address(signer)).
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'LinkedAppData',
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  // Asset that *is* a member of the collection.
  const memberAsset = await createAsset(umi, {
    collection: collection.publicKey,
  });

  const payloadText = 'member_write';
  const payload = new TextEncoder().encode(payloadText);

  await writeData(umi, {
    key: { type: 'LinkedAppData', dataAuthority },
    payer: signer,
    authority: signer,
    collection: collection.publicKey,
    asset: memberAsset.publicKey,
    data: payload,
  }).sendAndConfirm(umi);

  const fetched = await fetchAssetV1(umi, publicKey(memberAsset.publicKey));
  const section = (fetched.dataSections ?? []).find(
    (s) =>
      s.parentKey.type === 'LinkedAppData' &&
      s.parentKey.dataAuthority.type === 'Address' &&
      s.parentKey.dataAuthority.address === signer.publicKey
  );

  t.not(section, undefined, 'member asset should accept the write');
  t.is(
    new TextDecoder().decode(section!.data!),
    payloadText,
    'payload landed in the member asset bytes'
  );
});
