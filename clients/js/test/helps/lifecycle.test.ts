import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addressPluginAuthority,
  canBurn,
  canTransfer,
  pluginAuthorityPair,
} from '../../src';
import {
  createAsset,
  createAssetWithCollection,
  createUmi,
} from '../_setupRaw';

test('it can detect transferrable on basic asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  t.assert(canTransfer(owner.publicKey, asset));
});

test('it can detect non transferrable from frozen asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(!canTransfer(owner.publicKey, asset));
});

test('it can detect transferrable on asset with transfer delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  t.assert(canTransfer(delegate.publicKey, asset));
});

test('it can detect transferrable from permanent transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  t.assert(canTransfer(delegate.publicKey, asset));
});

test('it can detect transferrable when frozen with permanent transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(!canTransfer(owner.publicKey, asset));
  t.assert(canTransfer(delegate.publicKey, asset));
});

test('it can detect transferrable when frozen with permanent collection transfer delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
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
          type: 'PermanentTransferDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  t.assert(!canTransfer(owner.publicKey, asset, collection));
  t.assert(canTransfer(delegate.publicKey, asset, collection));
});

test('it can detect burnable on basic asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  t.assert(canBurn(owner.publicKey, asset));
});

test('it can detect non burnable from frozen asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.assert(!canBurn(owner.publicKey, asset));
});

test('it can detect burnable on asset with burn delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'BurnDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  t.assert(canBurn(delegate.publicKey, asset));
});

test('it can detect burnable from permanent burn', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentBurnDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  t.assert(canBurn(delegate.publicKey, asset));
});

test('it can detect burnable when frozen with permanent burn', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
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

  t.assert(!canBurn(owner.publicKey, asset));
  t.assert(canBurn(delegate.publicKey, asset));
});

test('it can detect burnable when frozen with permanent collection burn delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
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
          type: 'PermanentBurnDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  t.assert(!canBurn(owner.publicKey, asset, collection));
  t.assert(canBurn(delegate.publicKey, asset, collection));
});
