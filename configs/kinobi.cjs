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
    assetV1: {
      name: "baseAssetV1",
    },
    collectionV1: {
      name: "baseCollectionV1",
    }
  })
);

kinobi.update(new k.updateDefinedTypesVisitor({
  authority: {
    name: "pluginAuthority"
  }
}))

// Update instructions with default values
kinobi.update(
  k.updateInstructionsVisitor({
    // create: {
    //   bytesCreatedOnChain: k.bytesFromAccount("assetAccount"),
    // },
    transferV1: {
      arguments: {
        compressionProof: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    addPluginV1: {
      arguments: {
        initAuthority: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    addCollectionPluginV1: {
      arguments: {
        initAuthority: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    burnV1: {
      arguments: {
        compressionProof: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    createV1: {
      arguments: {
        plugins: {
          defaultValue: k.arrayValueNode([])
        },
        dataState: {
          defaultValue: k.enumValueNode('DataState', 'AccountState')
        }
      }
    },
    createCollectionV1: {
      arguments: {
        plugins: {
          defaultValue: k.noneValueNode()

        }
      }
    },
    collect: {
      accounts: {
        recipient: {
          defaultValue: k.publicKeyValueNode("8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v")
        }
      }
    }
  })
);

// Set ShankAccount discriminator.
const key = (name) => ({ field: "key", value: k.enumValueNode("Key", name) });
kinobi.update(
  k.setAccountDiscriminatorFromFieldVisitor({
    assetV1: key("AssetV1"),
    collectionV1: key("CollectionV1"),
  })
);

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);


// rewrite the account names for custom account data
kinobi.update(
  new k.updateAccountsVisitor({
    baseAssetV1: {
      name: "assetV1",
    },
    baseCollectionV1: {
      name: "collectionV1",
    }
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(k.renderJavaScriptVisitor(jsDir, {
  prettier,
  internalNodes: [],
  customAccountData: [{
    name: "assetV1",
    extract: true,
  }, {
    name: "collectionV1",
    extract: true,
  }],
}));