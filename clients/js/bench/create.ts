import { generateSigner, TransactionBuilder } from "@metaplex-foundation/umi";
import test from "ava";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { createCollectionV1, createV1, pluginAuthorityPair, ruleSet } from "../src";
import { createUmi } from "./_setup";

test('create a new, empty asset', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);

    const tx = await createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);
    const account = await umi.rpc.getAccount(assetAddress.publicKey);
    const space = account.exists ? account.data.length : 0;

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    const spaceResult = {
        name: `Space: ${t.title}`,
        unit: "Bytes",
        value: space,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    output.push(spaceResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('create a new, empty asset with empty collection', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);

    let builder = new TransactionBuilder();
    builder = builder.add(createCollectionV1(umi, {
        collection: collectionAddress,
        name: "Test",
        uri: "www.test.com",
    }));
    builder = builder.add(createV1(umi, {
        asset: assetAddress,
        collection: collectionAddress.publicKey,
        name: "Test",
        uri: "www.test.com",
    }));

    const tx = await builder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);
    const account = await umi.rpc.getAccount(assetAddress.publicKey);
    const space = account.exists ? account.data.length : 0;

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    const spaceResult = {
        name: `Space: ${t.title}`,
        unit: "Bytes",
        value: space,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    output.push(spaceResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('create a new asset with plugins', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);

    const tx = await createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
        plugins: [
            pluginAuthorityPair({
                type: 'Royalties',
                data: {
                    basisPoints: 5,
                    creators: [{ address: umi.identity.publicKey, percentage: 100 }],
                    ruleSet: ruleSet('None'),
                },
            }),
            pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: true } }),
            pluginAuthorityPair({ type: 'TransferDelegate' }),
            pluginAuthorityPair({ type: 'BurnDelegate' }),
        ],
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);
    const account = await umi.rpc.getAccount(assetAddress.publicKey);
    const space = account.exists ? account.data.length : 0;

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    const spaceResult = {
        name: `Space: ${t.title}`,
        unit: "Bytes",
        value: space,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    output.push(spaceResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('create a new asset with plugins and empty collection', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);

    let builder = new TransactionBuilder();
    builder = builder.add(createCollectionV1(umi, {
        collection: collectionAddress,
        name: "Test",
        uri: "www.test.com",
    }));
    builder = builder.add(createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
        plugins: [
            pluginAuthorityPair({
                type: 'Royalties',
                data: {
                    basisPoints: 5,
                    creators: [{ address: umi.identity.publicKey, percentage: 100 }],
                    ruleSet: ruleSet('None'),
                },
            }),
            pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: true } }),
            pluginAuthorityPair({ type: 'TransferDelegate' }),
            pluginAuthorityPair({ type: 'BurnDelegate' }),
        ],
    }));

    const tx = await builder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);
    const account = await umi.rpc.getAccount(assetAddress.publicKey);
    const space = account.exists ? account.data.length : 0;

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    const spaceResult = {
        name: `Space: ${t.title}`,
        unit: "Bytes",
        value: space,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    output.push(spaceResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});