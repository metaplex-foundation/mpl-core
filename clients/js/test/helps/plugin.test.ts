import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  AssetPluginKey,
  PluginType,
  addressPluginAuthority,
  assetPluginKeyFromType,
  checkPluginAuthorities,
  pluginAuthorityPair,
  pluginTypeFromAssetPluginKey,
  updatePluginAuthority,
} from '../../src';
import { createAssetWithCollection, createUmi } from '../_setupRaw';

test('it can convert plugin key to plugin type', async (t) => {
  const key: AssetPluginKey = 'royalties';
  t.is(pluginTypeFromAssetPluginKey(key), PluginType.Royalties);
});

test('it can convert plugin type to plugin key', async (t) => {
  const type: PluginType = PluginType.Royalties;
  t.is(assetPluginKeyFromType(type), 'royalties');
});

test('it can detect correct authorities on many types and plugins', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuth = generateSigner(umi);
  const transferDelegate = generateSigner(umi);
  const overrideDelegate = generateSigner(umi);
  const overridenDelegate = generateSigner(umi);
  const collectionDelegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      authority: updateAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
        pluginAuthorityPair({
          type: 'TransferDelegate',
          authority: addressPluginAuthority(transferDelegate.publicKey),
        }),
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: addressPluginAuthority(overrideDelegate.publicKey),
        }),
        pluginAuthorityPair({
          type: 'BurnDelegate',
          authority: updatePluginAuthority(),
        }),
      ],
    },
    {
      updateAuthority: updateAuth.publicKey,
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: addressPluginAuthority(overridenDelegate.publicKey),
        }),
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
          authority: addressPluginAuthority(collectionDelegate.publicKey),
        }),
      ],
    }
  );

  const checkUpdateAuth = checkPluginAuthorities({
    asset,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: updateAuth.publicKey,
    collection,
  });

  t.deepEqual(checkUpdateAuth, [false, false, false, true, false]);

  const checkTransferDelegate = checkPluginAuthorities({
    asset,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: transferDelegate.publicKey,
    collection,
  });

  t.deepEqual(checkTransferDelegate, [false, true, false, false, false]);

  const checkOverrideDelegate = checkPluginAuthorities({
    asset,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: overrideDelegate.publicKey,
    collection,
  });

  t.deepEqual(checkOverrideDelegate, [false, false, true, true, false]);

  const checkOverridenDelegate = checkPluginAuthorities({
    asset,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: overridenDelegate.publicKey,
    collection,
  });

  t.deepEqual(checkOverridenDelegate, [false, false, false, false, false]);

  const checkCollectionDelegate = checkPluginAuthorities({
    asset,
    pluginTypes: [
      PluginType.FreezeDelegate,
      PluginType.TransferDelegate,
      PluginType.UpdateDelegate,
      PluginType.BurnDelegate,
      PluginType.PermanentFreezeDelegate,
    ],
    authority: collectionDelegate.publicKey,
    collection,
  });

  t.deepEqual(checkCollectionDelegate, [false, false, false, false, true]);
});
