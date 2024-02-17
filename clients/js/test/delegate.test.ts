import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import { DataState, create, delegate, fetchAsset, getAssetAccountDataSerializer, getPluginHeaderSerializer, getPluginRegistrySerializer, } from '../src';
import { createUmi } from './_setup';

test('it initializes the plugin system correctly', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const delegateAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAsset(umi, assetAddress.publicKey);
  console.log("Before Plugins:\n", asset);
  const assetData = getAssetAccountDataSerializer().serialize(asset);

  await delegate(umi, {
    assetAddress: assetAddress.publicKey,
    owner: umi.identity,
    delegate: delegateAddress.publicKey,
  }).sendAndConfirm(umi);

  const pluginData = await umi.rpc.getAccount(assetAddress.publicKey);
  if (pluginData.exists) {
    const pluginHeader = getPluginHeaderSerializer().deserialize(pluginData.data, assetData.length);
    console.log("After Plugins:\n", pluginHeader);
    const pluginRegistry = getPluginRegistrySerializer().deserialize(pluginData.data, Number(pluginHeader[0].pluginMapOffset));
    console.log(pluginRegistry);
  } else {
    t.fail("Plugin data not found");
  }

  t.pass();
});
