import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { PluginType } from '../../src';
import {
  pluginTypeFromAssetPluginKey,
  assetPluginKeyFromType,
  checkPluginAuthorities,
  type AssetPluginKey,
} from '../../src/helpers/plugin';
import { pluginAuthorityPair } from '../../src/plugins';
import {
  createAssetWithCollection,
  createRpc,
  generateSignerWithSol,
} from '../_setup';

test('it can convert plugin key to plugin type', async (t) => {
  const key: AssetPluginKey = 'royalties';
  t.is(pluginTypeFromAssetPluginKey(key), PluginType.Royalties);
});

test('it can convert plugin type to plugin key', async (t) => {
  const type: PluginType = PluginType.Royalties;
  t.is(assetPluginKeyFromType(type), 'royalties');
});

test('it can detect correct authorities on many types and plugins', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const updateAuth = await generateKeyPairSigner();
  const transferDelegate = await generateKeyPairSigner();
  const overrideDelegate = await generateKeyPairSigner();
  const overridenDelegate = await generateKeyPairSigner();
  const collectionDelegate = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      owner: owner.address,
      authority: updateAuth,
      updateAuthority: updateAuth.address,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
        pluginAuthorityPair({
          type: 'TransferDelegate',
          authority: { __kind: 'Address', address: transferDelegate.address },
        }),
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          data: { additionalDelegates: [] },
          authority: { __kind: 'Address', address: overrideDelegate.address },
        }),
        pluginAuthorityPair({
          type: 'BurnDelegate',
          authority: { __kind: 'UpdateAuthority' },
        }),
      ],
    },
    {
      updateAuthority: updateAuth.address,
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          data: { additionalDelegates: [] },
          authority: { __kind: 'Address', address: overridenDelegate.address },
        }),
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
          authority: { __kind: 'Address', address: collectionDelegate.address },
        }),
      ],
    }
  );

  const checkUpdateAuth = checkPluginAuthorities({
    asset: asset.data as any,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: updateAuth.address,
    collection: collection.data as any,
  });

  t.deepEqual(checkUpdateAuth, [false, false, false, true, false]);

  const checkTransferDelegate = checkPluginAuthorities({
    asset: asset.data as any,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: transferDelegate.address,
    collection: collection.data as any,
  });

  t.deepEqual(checkTransferDelegate, [false, true, false, false, false]);

  const checkOverrideDelegate = checkPluginAuthorities({
    asset: asset.data as any,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: overrideDelegate.address,
    collection: collection.data as any,
  });

  t.deepEqual(checkOverrideDelegate, [false, false, true, true, false]);

  const checkOverridenDelegate = checkPluginAuthorities({
    asset: asset.data as any,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: overridenDelegate.address,
    collection: collection.data as any,
  });

  t.deepEqual(checkOverridenDelegate, [false, false, false, false, false]);

  const checkCollectionDelegate = checkPluginAuthorities({
    asset: asset.data as any,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: collectionDelegate.address,
    collection: collection.data as any,
  });

  t.deepEqual(checkCollectionDelegate, [false, false, false, false, true]);
});
