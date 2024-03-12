const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instanciate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "mpl_core.json")]);

// Update programs.
kinobi.update(
  k.updateProgramsVisitor({
    mplCoreProgram: { name: "mplCore" },
  })
);

// Set default account values across multiple instructions.
// kinobi.update(
//   new k.SetInstructionAccountDefaultValuesVisitor([
//     {
//       account: "logWrapper",
//       defaultsTo: k.conditionalDefault("arg", "dataState", {
//         value: k.vEnum("DataState", "LedgerState"),
//         ifTrue: k.programDefault(
//           "splNoop",
//           "noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV"
//         ),
//       }),
//     },
//   ])
// );

// // Update accounts.
// kinobi.update(
//   new k.UpdateAccountsVisitor({
//     myPdaAccount: {
//       seeds: [
//         k.stringConstantSeed("myPdaAccount"),
//         k.programSeed(),
//         k.publicKeySeed("authority", "The address of the authority"),
//         k.stringSeed("name", "The name of the account"),
//       ],
//     },
//   })
// );

kinobi.update(
  new k.updateAccountsVisitor({
    asset: {
      name: "baseAsset",
    },
    collection: {
      name: "baseCollection",
    }
  })
);

// Update instructions with default values
kinobi.update(
  k.updateInstructionsVisitor({
    // create: {
    //   bytesCreatedOnChain: k.bytesFromAccount("assetAccount"),
    // },
    transfer: {
      arguments: {
        compressionProof: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    addPlugin: {
      arguments: {
        initAuthority: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    addCollectionPlugin: {
      arguments: {
        initAuthority: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    burn: {
      arguments: {
        compressionProof: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    create: {
      arguments: {
        plugins: {
          defaultValue: k.arrayValueNode([])
        }
      }
    },
    createCollection: {
      arguments: {
        plugins: {
          defaultValue: k.arrayValueNode([])

        }
      }
    }
  })
);

// Set ShankAccount discriminator.
const key = (name) => ({ field: "key", value: k.enumValueNode("Key", name) });
kinobi.update(
  k.setAccountDiscriminatorFromFieldVisitor({
    asset: key("Asset"),
    collection: key("Collection"),
    // myPdaAccount: key("MyPdaAccount"),
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(k.renderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
