import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { createV2, createCollection, fetchAssetV1, mplCore, addCollectionPlugin, updateV2, updateAuthority, addPlugin } from "@metaplex-foundation/mpl-core";
import { createSignerFromKeypair, generateSigner, signerIdentity, sol } from "@metaplex-foundation/umi";
//import { Keypair, PublicKey } from "@solana/web3.js";

import wallet from "../../../wallet.json"

const umi = createUmi("http://127.0.0.1:8899").use(mplCore());

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(signer));
// Works
const test1 = async () => {
    //const umi = await createUmi();
    const assetOwner = umi.identity;
    //const collectionUmi = await createUmi();

    const assetSigner = generateSigner(umi);
    await umi.rpc.airdrop(keypair.publicKey, sol(10));
    //await umi.rpc.airdrop(assetSigner.publicKey, sol(10));
    const assetAddress = assetSigner.publicKey;
    await createV2(umi, {
      asset: assetSigner,
      name: 'My Asset',
      uri: 'https://example.com/my-asset.json',
    }).sendAndConfirm(umi);
  
    const collectionSigner = generateSigner(umi);
    const collectionAddress = collectionSigner.publicKey;
    //expect(false).toBeTruthy()
    await createCollection(umi, {
      collection: collectionSigner,
      name: 'My Collection',
      uri: 'https://example.com/my-collection.json',
    }).sendAndConfirm(umi);
  
    await addCollectionPlugin(umi, {
      collection: collectionAddress,
      plugin: {
        type: 'UpdateDelegate',
        authority: { type: 'Address', address: assetOwner.publicKey },
        additionalDelegates: [],
      },
    }).sendAndConfirm(umi);
  
    await updateV2(umi, {
      asset: assetAddress,
      newCollection: collectionAddress,
      newUpdateAuthority: updateAuthority('Collection', [collectionAddress]),
    }).sendAndConfirm(umi);
  
    const asset = await fetchAssetV1(umi, assetAddress);
    console.log("\n\n\nTest 1 -> Asset Owner matches? ", asset.owner === assetOwner.publicKey);

    console.log("Test 1 -> Update Authority matches? ", asset.updateAuthority.type === 'Collection' && asset.updateAuthority.address === collectionAddress);
};

const test2 = async() => {
  const umi = createUmi("http://127.0.0.1:8899").use(mplCore());
  let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
  const signer = createSignerFromKeypair(umi, keypair);
  umi.use(signerIdentity(signer));

  const assetOwner = umi.identity;

  const collectionUmi = createUmi("http://127.0.0.1:8899").use(mplCore());
  let collectionUmiKeypair = umi.eddsa.generateKeypair();
  await umi.rpc.airdrop(collectionUmiKeypair.publicKey, sol(10));
  const collectionUmiSigner = createSignerFromKeypair(umi, collectionUmiKeypair);
  collectionUmi.use(signerIdentity(collectionUmiSigner));

  const assetSigner = generateSigner(umi);
  const assetAddress = assetSigner.publicKey;
  await createV2(umi, {
    asset: assetSigner,
    name: 'My Asset',
    uri: 'https://example.com/my-asset.json',
  }).sendAndConfirm(umi);

  const collectionSigner = generateSigner(collectionUmi);
  const collectionAddress = collectionSigner.publicKey;
  await createCollection(collectionUmi, {
    collection: collectionSigner,
    name: 'My Collection',
    uri: 'https://example.com/my-collection.json',
  }).sendAndConfirm(collectionUmi);

  await addCollectionPlugin(collectionUmi, {
    collection: collectionAddress,
    plugin: {
      type: 'UpdateDelegate',
      // Some other address
      authority: { type: 'Address', address: generateSigner(umi).publicKey },
      // Who we want to be able to add collection assets
      additionalDelegates: [assetOwner.publicKey],
    },
  }).sendAndConfirm(collectionUmi);

  await updateV2(umi, {
    asset: assetAddress,
    authority: assetOwner,
    newCollection: collectionAddress,
    newUpdateAuthority: updateAuthority('Collection', [collectionAddress]),
  }).sendAndConfirm(umi);

  const asset = await fetchAssetV1(umi, assetAddress);
  console.log("\n\n\nTest 2 -> Asset owner: ", asset.owner)
  console.log("Test 2 -> Asset Owner matches? ", asset.owner === assetOwner.publicKey);

  console.log("Test 2 -> Update Authority type: ", asset.updateAuthority.type);
  console.log("Test 2 -> Update Authority address: ", asset.updateAuthority.address);
  console.log("Test 2 -> Update Authority matches? ", asset.updateAuthority.type === 'Collection' && asset.updateAuthority.address === collectionAddress);
}

const test3 = async() => {
  const umi = createUmi("http://127.0.0.1:8899").use(mplCore());
  let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
  const signer = createSignerFromKeypair(umi, keypair);
  umi.use(signerIdentity(signer));

  const assetOwner = umi.identity;

  const collectionUmi = createUmi("http://127.0.0.1:8899").use(mplCore());
  let collectionUmiKeypair = umi.eddsa.generateKeypair();
  await umi.rpc.airdrop(collectionUmiKeypair.publicKey, sol(10));
  const collectionUmiSigner = createSignerFromKeypair(umi, collectionUmiKeypair);
  collectionUmi.use(signerIdentity(collectionUmiSigner));
  const collectionOwner = collectionUmi.identity;

  const assetSigner = generateSigner(umi);
  await createV2(umi, {
    asset: assetSigner,
    name: 'My Asset',
    uri: 'https://example.com/my-asset.json',
  }).sendAndConfirm(umi);

  const collectionSigner = generateSigner(collectionUmi);
  const collectionAddress = collectionSigner.publicKey;
  await createCollection(collectionUmi, {
    collection: collectionSigner,
    name: 'My Collection',
    uri: 'https://example.com/my-collection.json',
  }).sendAndConfirm(collectionUmi);

  await addPlugin(umi, {
    asset: assetSigner.publicKey,
    plugin: {
      type: 'UpdateDelegate',
      authority: { type: 'Address', address: collectionOwner.publicKey },
      additionalDelegates: [],
    },
  }).sendAndConfirm(umi);

  await updateV2(collectionUmi, {
    asset: assetSigner.publicKey,
    authority: collectionOwner,
    newCollection: collectionAddress,
    newUpdateAuthority: updateAuthority('Collection', [collectionAddress]),
  }).sendAndConfirm(collectionUmi);

  const asset = await fetchAssetV1(umi, assetSigner.publicKey);
  console.log("\n\n\nTest 3 -> Asset owner: ", asset.owner)
  console.log("Test 3 -> Asset Owner matches? ", asset.owner === assetOwner.publicKey);

  console.log("Test 3 -> Update Authority type: ", asset.updateAuthority.type);
  console.log("Test 3 -> Update Authority address: ", asset.updateAuthority.address);
  console.log("Test 3 -> Update Authority matches? ", asset.updateAuthority.type === 'Collection' && asset.updateAuthority.address === collectionAddress);
}

test1().then(() => {
    console.log("\nTest 1 completed!")
});

test2().then(() => {
    console.log("\nTest 2 completed!")
})

test3().then(() => {
    console.log("\nTest 3 completed!")
})

