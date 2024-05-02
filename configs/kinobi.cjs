const path = require("path");
const k = require("../../kinobi/dist/cjs");

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
        recipient1: {
          defaultValue: k.publicKeyValueNode("8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v")
        },
        recipient2: {
          defaultValue: k.publicKeyValueNode("MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt")
        }
      }
    },
    updateV1: {
      arguments: {
        newUpdateAuthority: {
          defaultValue: k.noneValueNode()
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
  }, {
    name: "pluginRegistryV1",
    extract: true,
  }],
}));