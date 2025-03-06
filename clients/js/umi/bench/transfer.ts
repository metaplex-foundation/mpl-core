import { generateSigner, TransactionBuilder } from "@metaplex-foundation/umi";
import test from "ava";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { createCollectionV1, createV1, pluginAuthorityPair, ruleSet, transferV1 } from "../src";
import { createUmi } from "./_setup";

test('transfer an empty asset', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);

    await createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
    }).sendAndConfirm(umi);

    const tx = await transferV1(umi, {
        asset: assetAddress.publicKey,
        newOwner: newOwner.publicKey,
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('transfer an empty asset with empty collection', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);

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

    await builder.sendAndConfirm(umi);

    const tx = await transferV1(umi, {
        asset: assetAddress.publicKey,
        newOwner: newOwner.publicKey,
        collection: collectionAddress.publicKey,
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('transfer an asset with plugins', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);

    await createV1(umi, {
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
            pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
            pluginAuthorityPair({ type: 'TransferDelegate' }),
            pluginAuthorityPair({ type: 'BurnDelegate' }),
        ],
    }).sendAndConfirm(umi);

    const tx = await transferV1(umi, {
        asset: assetAddress.publicKey,
        newOwner: newOwner.publicKey,
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('transfer an asset with plugins and empty collection', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);

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
        plugins: [
            pluginAuthorityPair({
                type: 'Royalties',
                data: {
                    basisPoints: 5,
                    creators: [{ address: umi.identity.publicKey, percentage: 100 }],
                    ruleSet: ruleSet('None'),
                },
            }),
            pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
            pluginAuthorityPair({ type: 'TransferDelegate' }),
            pluginAuthorityPair({ type: 'BurnDelegate' }),
        ],
    }));

    await builder.sendAndConfirm(umi);

    const tx = await transferV1(umi, {
        asset: assetAddress.publicKey,
        newOwner: newOwner.publicKey,
        collection: collectionAddress.publicKey,
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);
    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});