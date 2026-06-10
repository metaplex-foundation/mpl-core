import test from 'ava';
import { isFrozen } from '../../src/helpers/state';
import { pluginAuthorityPair } from '../../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createRpc,
  generateSignerWithSol,
} from '../_setup';

test('it can detect frozen from freeze delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(isFrozen(asset.data as any));
});

test('it can detect unfrozen from freeze delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  t.assert(!isFrozen(asset.data as any));
});

test('it can detect frozen from permanent freeze delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(isFrozen(asset.data as any));
});

test('it can detect unfrozen from permanent freeze delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  t.assert(!isFrozen(asset.data as any));
});

test('it can detect frozen from permanent freeze on collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
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

  t.assert(isFrozen(asset.data as any, collection.data as any));
});

test('it can detect unfrozen from freeze on asset and permanent freeze on collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: false },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
        }),
      ],
    }
  );

  t.assert(!isFrozen(asset.data as any, collection.data as any));
});

test('it can detect frozen from freeze on asset and permanent freeze on collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: false },
        }),
      ],
    }
  );

  t.assert(isFrozen(asset.data as any, collection.data as any));
});
