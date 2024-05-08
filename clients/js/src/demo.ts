// @ts-nocheck
import { createUmi, generateSigner } from '@metaplex-foundation/umi';
import {
  create,
  createCollection,
  fetchAssetV1,
  getAssetV1GpaBuilder,
  ruleSet,
  transfer,
  updateAuthority,
  Key,
  fetchCollectionV1,
  addPlugin,
  revokePluginAuthority,
} from './index';

const example = async () => {
  const umi = await createUmi();

  // Create an asset
  const assetAddress = generateSigner(umi);
  const owner = generateSigner(umi);

  await create(umi, {
    name: 'Test Asset',
    uri: 'https://example.com/asset.json',
    asset: assetAddress,
    owner: owner.publicKey, // optional, will default to payer
  }).sendAndConfirm(umi);

  // Fetch an asset
  const asset = await fetchAssetV1(umi, assetAddress.publicKey);

  // Create a collection
  const collectionUpdateAuthority = generateSigner(umi);
  const collectionAddress = generateSigner(umi);
  await createCollection(umi, {
    name: 'Test Collection',
    uri: 'https://example.com/collection.json',
    collection: collectionAddress,
    updateAuthority: collectionUpdateAuthority.publicKey, // optional, defaults to payer
  }).sendAndConfirm(umi);

  // Fetch a collection
  const collection = await fetchCollectionV1(umi, collectionAddress.publicKey);

  // Create an asset in a collection, the authority must be the updateAuthority of the collection
  await create(umi, {
    name: 'Test Asset',
    uri: 'https://example.com/asset.json',
    asset: assetAddress,
    collection,
    authority: collectionUpdateAuthority, // optional, defaults to payer
  }).sendAndConfirm(umi);

  // Transfer an asset
  const recipient = generateSigner(umi);
  await transfer(umi, {
    asset,
    newOwner: recipient.publicKey,
  }).sendAndConfirm(umi);

  // Transfer an asset in a collection
  await transfer(umi, {
    asset,
    newOwner: recipient.publicKey,
    collection,
  }).sendAndConfirm(umi);

  // GPA fetch assets by owner
  const assetsByOwner = await getAssetV1GpaBuilder(umi)
    .whereField('key', Key.AssetV1)
    .whereField('owner', owner.publicKey)
    .getDeserialized();

  // GPA fetch assets by collection
  const assetsByCollection = await getAssetV1GpaBuilder(umi)
    .whereField('key', Key.AssetV1)
    .whereField(
      'updateAuthority',
      updateAuthority('Collection', [collectionAddress.publicKey])
    )
    .getDeserialized();

  // DAS API (RPC based indexing) fetch assets by owner/collection
  // Coming soon
};

const advancedExamples = async () => {
  const umi = await createUmi();

  // Freezing an asset
  const assetAddress = generateSigner(umi);
  const freezeDelegate = generateSigner(umi);

  await addPlugin(umi, {
    asset: assetAddress.publicKey,
    // adds the owner-managed freeze plugin to the asset
    plugin: {
      type: 'FreezeDelegate',
      frozen: true,

      // Optionally set the authority to a delegate who can unfreeze. If unset, this will be the Owner
      // This is functionally the same as calling addPlugin and approvePluginAuthority separately.
      // Freezing with a delegate is commonly used for escrowless staking programs.
      authority: {
        type: 'Address',
        address: freezeDelegate.publicKey,
      },
    },
  }).sendAndConfirm(umi);

  // Unfreezing an asset with a delegate
  // Revoking an authority will revert the authority back to the owner for owner-managed plugins
  await revokePluginAuthority(umi, {
    asset: assetAddress.publicKey,
    plugin: {
      type: 'FreezeDelegate',
    },
    authority: freezeDelegate,
  }).sendAndConfirm(umi);

  // Create a collection with royalties
  const collectionAddress = generateSigner(umi);
  const creator1 = generateSigner(umi);
  const creator2 = generateSigner(umi);

  await createCollection(umi, {
    name: 'Test Collection',
    uri: 'https://example.com/collection.json',
    collection: collectionAddress,
    plugins: [
      {
        type: 'Royalties',
        basisPoints: 500,
        creators: [
          {
            address: creator1.publicKey,
            percentage: 20,
          },
          {
            address: creator2.publicKey,
            percentage: 80,
          },
        ],
        ruleSet: ruleSet('None'), // Compatibility rule set
      },
    ],
  }).sendAndConfirm(umi);

  // Create an asset in a collection.
  // Assets in a collection will inherit the collection's authority-managed plugins, in this case the royalties plugin
  await create(umi, {
    name: 'Test Asset',
    uri: 'https://example.com/asset.json',
    asset: assetAddress,
    collection: await fetchCollectionV1(umi, collectionAddress.publicKey),
  }).sendAndConfirm(umi);
};
