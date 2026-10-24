import assert from 'node:assert/strict';
import test from 'node:test';
import {
  airdropFactory,
  appendTransactionMessageInstructions,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  generateKeyPairSigner,
  lamports,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from '@solana/kit';
import * as core from '../../dist/index.js';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'http://127.0.0.1:8899');
const rpcSubscriptions = createSolanaRpcSubscriptions(
  process.env.WS_URL ?? 'ws://127.0.0.1:8900'
);
const sendAndConfirm = sendAndConfirmTransactionFactory({
  rpc,
  rpcSubscriptions,
});

async function payer() {
  const signer = await generateKeyPairSigner();
  await airdropFactory({ rpc, rpcSubscriptions })({
    commitment: 'confirmed',
    lamports: lamports(2_000_000_000n),
    recipientAddress: signer.address,
  });
  return signer;
}

async function send(signer, ...instructions) {
  const { value: blockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(signer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstructions(instructions, m)
  );
  const transaction = await signTransactionMessageWithSigners(message);
  await sendAndConfirm(transaction, { commitment: 'confirmed' });
}

const plugin = (kind, value) => ({ __kind: kind, fields: [value] });
const addressAuthority = (signer) => ({
  __kind: 'Address',
  address: signer.address,
});
const freeze = (frozen) => plugin('FreezeDelegate', { frozen });
const attributes = (value) =>
  plugin('Attributes', { attributeList: [{ key: 'level', value }] });

// Tests exercise only public package exports against the current locally built program.
test('asset and collection lifecycle, authorities, and internal plugins', async () => {
  const authority = await payer();
  const asset = await generateKeyPairSigner();
  const collection = await generateKeyPairSigner();
  const delegate = await generateKeyPairSigner();
  const recipient = await generateKeyPairSigner();

  await send(
    authority,
    core.getCreateCollectionV2Instruction({
      collection,
      name: 'Kit collection',
      payer: authority,
      plugins: [{ authority: null, plugin: attributes('collection') }],
      updateAuthority: authority.address,
      uri: 'https://example.com/collection.json',
    })
  );
  const fetchedCollection = await core.fetchCollectionV1(
    rpc,
    collection.address
  );
  assert.equal(fetchedCollection.data.name, 'Kit collection');
  assert.deepEqual(
    fetchedCollection.data.plugins[0].plugin,
    attributes('collection')
  );

  await send(
    authority,
    core.getCreateV2Instruction({
      asset,
      authority,
      collection: collection.address,
      name: 'Kit asset',
      owner: authority.address,
      payer: authority,
      plugins: [{ authority: null, plugin: freeze(false) }],
      uri: 'https://example.com/asset.json',
    })
  );
  let fetched = await core.fetchAssetV1(rpc, asset.address);
  assert.deepEqual(
    fetched.data.updateAuthority,
    plugin('Collection', collection.address)
  );
  assert.deepEqual(fetched.data.plugins[0].plugin, freeze(false));
  assert.equal(fetched.data.owner, authority.address);
  assert.equal(
    (await core.fetchCollectionV1(rpc, collection.address)).data.currentSize,
    1
  );

  const common = {
    asset: asset.address,
    collection: collection.address,
    payer: authority,
  };
  await send(
    authority,
    core.getAddPluginV1Instruction({ ...common, plugin: attributes('one') })
  );
  await send(
    authority,
    core.getUpdatePluginV1Instruction({ ...common, plugin: attributes('two') })
  );
  fetched = await core.fetchAssetV1(rpc, asset.address);
  assert.deepEqual(
    fetched.data.plugins.find((p) => p.plugin.__kind === 'Attributes').plugin,
    attributes('two')
  );

  await send(
    authority,
    core.getApprovePluginAuthorityV1Instruction({
      ...common,
      newAuthority: addressAuthority(delegate),
      pluginType: core.PluginType.FreezeDelegate,
    })
  );
  fetched = await core.fetchAssetV1(rpc, asset.address);
  assert.deepEqual(
    fetched.data.plugins.find((p) => p.plugin.__kind === 'FreezeDelegate')
      .authority,
    addressAuthority(delegate)
  );
  await send(
    authority,
    core.getUpdatePluginV1Instruction({
      ...common,
      authority: delegate,
      plugin: freeze(true),
    })
  );
  assert.deepEqual(
    (await core.fetchAssetV1(rpc, asset.address)).data.plugins.find(
      (p) => p.plugin.__kind === 'FreezeDelegate'
    ).plugin,
    freeze(true)
  );
  await assert.rejects(
    send(
      authority,
      core.getTransferV1Instruction({ ...common, newOwner: recipient.address })
    )
  );
  assert.equal(
    (await core.fetchAssetV1(rpc, asset.address)).data.owner,
    authority.address
  );
  await send(
    authority,
    core.getUpdatePluginV1Instruction({
      ...common,
      authority: delegate,
      plugin: freeze(false),
    })
  );
  await send(
    authority,
    core.getRevokePluginAuthorityV1Instruction({
      ...common,
      pluginType: core.PluginType.FreezeDelegate,
    })
  );
  await send(
    authority,
    core.getRemovePluginV1Instruction({
      ...common,
      pluginType: core.PluginType.Attributes,
    })
  );
  assert.equal(
    (await core.fetchAssetV1(rpc, asset.address)).data.plugins.length,
    1
  );

  await send(
    authority,
    core.getUpdateV2Instruction({ ...common, newName: 'Updated asset' })
  );
  assert.equal(
    (await core.fetchAssetV1(rpc, asset.address)).data.name,
    'Updated asset'
  );
  await send(
    authority,
    core.getUpdateCollectionV1Instruction({
      collection: collection.address,
      newName: 'Updated collection',
      payer: authority,
    })
  );
  assert.equal(
    (await core.fetchCollectionV1(rpc, collection.address)).data.name,
    'Updated collection'
  );

  await send(
    authority,
    core.getTransferV1Instruction({ ...common, newOwner: recipient.address })
  );
  assert.equal(
    (await core.fetchAssetV1(rpc, asset.address)).data.owner,
    recipient.address
  );
  assert.equal(
    (await core.fetchAllAssetV1(rpc, [asset.address]))[0].data.name,
    'Updated asset'
  );
  await send(
    authority,
    core.getBurnV1Instruction({ ...common, authority: recipient })
  );
  const burned = await rpc
    .getAccountInfo(asset.address, { encoding: 'base64' })
    .send();
  assert.ok(
    burned.value === null ||
      Buffer.from(burned.value.data[0], 'base64')[0] === core.Key.Uninitialized
  );
  assert.equal(
    (await core.fetchCollectionV1(rpc, collection.address)).data.currentSize,
    0
  );
  await send(
    authority,
    core.getBurnCollectionV1Instruction({
      collection: collection.address,
      compressionProof: null,
      payer: authority,
    })
  );
});

test('ordinary fetch decodes external AppData on assets and collections', async () => {
  const authority = await payer();
  const dataAuthority = addressAuthority(authority);
  for (const isCollection of [false, true]) {
    const account = await generateKeyPairSigner();
    const common = { payer: authority };
    const create = isCollection
      ? core.getCreateCollectionV2Instruction({
          ...common,
          collection: account,
          name: 'External collection',
          uri: '',
        })
      : core.getCreateV2Instruction({
          ...common,
          asset: account,
          name: 'External asset',
          uri: '',
        });
    await send(authority, create);
    const accountInput = isCollection
      ? { collection: account.address }
      : { asset: account.address };
    const add = isCollection
      ? core.getAddCollectionExternalPluginAdapterV1Instruction
      : core.getAddExternalPluginAdapterV1Instruction;
    const write = isCollection
      ? core.getWriteCollectionExternalPluginAdapterDataV1Instruction
      : core.getWriteExternalPluginAdapterDataV1Instruction;
    const remove = isCollection
      ? core.getRemoveCollectionExternalPluginAdapterV1Instruction
      : core.getRemoveExternalPluginAdapterV1Instruction;
    const fetch = isCollection ? core.fetchCollectionV1 : core.fetchAssetV1;
    const key = plugin('AppData', dataAuthority);
    await send(
      authority,
      add({
        ...accountInput,
        ...common,
        initInfo: plugin('AppData', {
          dataAuthority,
          initPluginAuthority: null,
          schema: core.ExternalPluginAdapterSchema.Json,
        }),
      })
    );
    await send(
      authority,
      write({
        ...accountInput,
        ...common,
        data: new TextEncoder().encode('{"level":5}'),
        key,
      })
    );
    const fetched = await fetch(rpc, account.address);
    assert.equal(fetched.data.externalPluginAdapters.length, 1);
    assert.equal(
      fetched.data.externalPluginAdapters[0].adapter.__kind,
      'AppData'
    );
    assert.deepEqual(fetched.data.externalPluginAdapters[0].data, { level: 5 });
    // Data shrinks while an internal plugin trails the external payload.
    const addPlugin = isCollection
      ? core.getAddCollectionPluginV1Instruction
      : core.getAddPluginV1Instruction;
    await send(
      authority,
      addPlugin({ ...accountInput, ...common, plugin: attributes('preserved') })
    );
    await send(
      authority,
      write({
        ...accountInput,
        ...common,
        data: new TextEncoder().encode('{}'),
        key,
      })
    );
    const shrunk = await fetch(rpc, account.address);
    assert.deepEqual(shrunk.data.externalPluginAdapters[0].data, {});
    assert.deepEqual(shrunk.data.plugins[0].plugin, attributes('preserved'));
    await send(authority, remove({ ...accountInput, ...common, key }));
    assert.deepEqual(
      (await fetch(rpc, account.address)).data.externalPluginAdapters,
      []
    );
  }
});
