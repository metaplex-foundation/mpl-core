import test from 'ava';
import { isFrozen, pluginAuthorityPair } from '../../src';
import {
  createAsset,
  createAssetWithCollection,
  createUmi,
} from '../_setupRaw';

test('it can detect frozen from freeze delegate', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(isFrozen(asset));
});

test('it can detect unfrozen from freeze delegate', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  t.assert(!isFrozen(asset));
});

test('it can detect frozen from permanent freeze delegate', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(isFrozen(asset));
});

test('it can detect unfrozen from permanent freeze delegate', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  t.assert(!isFrozen(asset));
});

test('it can detect frozen from permanent freeze on collection', async (t) => {
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

  t.assert(isFrozen(asset, collection));
});

test('it can detect unfrozen from freeze on asset and permanent freeze on collection', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(
    umi,
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

  t.assert(!isFrozen(asset, collection));
});

test('it can detect frozen from freeze on asset and permanent freeze on collection', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(
    umi,
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

  t.assert(isFrozen(asset, collection));
});
