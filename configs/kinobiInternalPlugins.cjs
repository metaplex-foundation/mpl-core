/**
 * Custom Kinobi renderer for the mpl-core internal (first-party) plugin type
 * layer — the content that used to be hand-written in clients/js/src/plugins/types.ts.
 *
 * Adding an internal plugin used to mean editing types.ts in ~6 places: a
 * `XPlugin` alias, membership in the right `*PluginArgsV2` union, and membership
 * in `Common/Asset/CollectionPluginsList`. This renderer derives all of that:
 *
 *   - the plugin set + whether each carries data comes from the IDL `Plugin` enum,
 *   - the owner-managed / authority-managed / create-only split comes from the
 *     program itself via configs/plugin-manifest.json (emitted by the Rust
 *     `dump_plugin_manifest` example from `PluginType::manager()` and the
 *     permanent-delegate set), so the unions cannot drift from on-chain rules,
 *   - the asset/collection/common *list* scope is an SDK deserialization
 *     convention (deliberately more permissive than the program's create-time
 *     rules — e.g. owner-managed plugins are listed as common), so it lives in
 *     the small SCOPE table below.
 *
 * A brand-new common internal plugin needs no changes here at all. An
 * asset-only or collection-only plugin needs a one-line SCOPE entry; a plugin
 * with a bespoke ergonomic wrapper (like Royalties/MasterEdition) needs an
 * OVERRIDES entry.
 */

const fs = require('fs');
const path = require('path');
const { pascalCase, camelCase } = require('@metaplex-foundation/kinobi');
const { makeImports, header, writeRenderMap } = require('./kinobiRenderUtils.cjs');

const HEADER = header(
  'kinobiInternalPlugins.cjs',
  ' (plugin set +\n * configs/plugin-manifest.json)'
);

const IMPORT_ORDER = [
  '..',
  '../../plugins/types',
  '../../plugins/pluginAuthority',
  '../../plugins/royalties',
  '../../plugins/masterEdition',
];

// SDK list scope (fetch-side). Anything not listed here is "common" (present on
// both assets and collections). This is an SDK convention, not a program rule.
const SCOPE = {
  asset: ['FreezeDelegate', 'BurnDelegate', 'TransferDelegate', 'Edition'],
  collection: ['MasterEdition', 'BubblegumV2'],
};

// Plugins that can appear on a GroupV1 account.
const GROUP_PLUGINS = ['Attributes', 'Autograph', 'VerifiedCreators'];

// Plugins whose ergonomic data/args types are hand-written wrappers rather than
// the generated `X`/`XArgs` (because the base type is remapped and wrapped).
const OVERRIDES = {
  Royalties: {
    dataType: { name: 'Royalties', from: '../../plugins/royalties' },
    argsType: { name: 'RoyaltiesArgs', from: '../../plugins/royalties' },
    // RoyaltiesPlugin is defined in royalties.ts; import rather than generate.
    pluginType: { name: 'RoyaltiesPlugin', from: '../../plugins/royalties' },
  },
  MasterEdition: {
    dataType: { name: 'MasterEdition', from: '../../plugins/masterEdition' },
    argsType: { name: 'MasterEditionArgs', from: '../../plugins/masterEdition' },
  },
};

function buildPlugins(root, manifest) {
  const program = root.programs[0];
  const dtByName = new Map();
  program.definedTypes.forEach((dt) => dtByName.set(String(dt.name), dt));

  const pluginEnum = dtByName.get('plugin');
  if (!pluginEnum || pluginEnum.type.kind !== 'enumTypeNode') {
    throw new Error('Could not find `plugin` enum in the IDL.');
  }

  const byName = {};
  manifest.forEach((m) => {
    byName[m.name] = m;
  });

  return pluginEnum.type.variants.map((v) => {
    const pascal = pascalCase(String(v.name));
    const meta = byName[pascal];
    if (!meta) {
      throw new Error(
        `Plugin "${pascal}" is in the IDL but missing from plugin-manifest.json`
      );
    }
    // Resolve the variant's inner defined type to learn whether it carries data.
    let innerLink = null;
    if (v.tuple && v.tuple.items && v.tuple.items[0]) innerLink = v.tuple.items[0];
    else if (v.fields && v.fields[0]) innerLink = v.fields[0].type || v.fields[0];
    const innerName =
      innerLink && innerLink.kind === 'definedTypeLinkNode'
        ? String(innerLink.name)
        : null;
    const innerDt = innerName ? dtByName.get(innerName) : null;
    const hasData =
      !!innerDt &&
      innerDt.type.kind === 'structTypeNode' &&
      innerDt.type.fields.length > 0;

    const ov = OVERRIDES[pascal] || {};
    return {
      pascal,
      camel: camelCase(pascal),
      manager: meta.manager,
      createOnly: meta.createOnly,
      hasData,
      dataType: ov.dataType || { name: pascal, from: '..' },
      argsType: ov.argsType || { name: `${pascal}Args`, from: '..' },
      pluginType: ov.pluginType || null, // null => generate `${pascal}Plugin`
      scope: SCOPE.asset.includes(pascal)
        ? 'asset'
        : SCOPE.collection.includes(pascal)
          ? 'collection'
          : 'common',
    };
  });
}

function generateInternalFile(plugins, imp) {
  const argMember = (p) => {
    if (!p.hasData) return `{\n      type: '${p.pascal}';\n    }`;
    imp.use(p.argsType.from, p.argsType.name);
    return `({\n      type: '${p.pascal}';\n    } & ${p.argsType.name})`;
  };

  const union = (name, members) =>
    `export type ${name} =\n  | ${members.map(argMember).join('\n  | ')};`;

  const createOnly = plugins.filter((p) => p.createOnly);
  const ownerManaged = plugins.filter((p) => !p.createOnly && p.manager === 'Owner');
  const authorityManaged = plugins.filter(
    (p) => !p.createOnly && p.manager !== 'Owner'
  );

  // Plugin type aliases.
  const pluginTypeName = (p) => {
    if (p.pluginType) {
      imp.use(p.pluginType.from, p.pluginType.name);
      return p.pluginType.name;
    }
    return `${p.pascal}Plugin`;
  };
  const pluginAliases = plugins
    .filter((p) => !p.pluginType) // Royalties is imported, not generated
    .map((p) => {
      imp.use(p.dataType.from, p.dataType.name);
      return `export type ${p.pascal}Plugin = BasePlugin & ${p.dataType.name};`;
    })
    .join('\n');

  const listEntry = (p) => `  ${p.camel}?: ${pluginTypeName(p)};`;
  const commonList = plugins.filter((p) => p.scope === 'common');
  const assetList = plugins.filter((p) => p.scope === 'asset');
  const collectionList = plugins.filter((p) => p.scope === 'collection');
  const groupList = plugins.filter((p) => GROUP_PLUGINS.includes(p.pascal));

  imp.use('../../plugins/types', 'BasePlugin');
  imp.use('../../plugins/pluginAuthority', 'PluginAuthority');

  const body = [
    union('CreateOnlyPluginArgsV2', createOnly),
    '',
    union('OwnerManagedPluginArgsV2', ownerManaged),
    '',
    union('AuthorityManagedPluginArgsV2', authorityManaged),
    '',
    `export type AuthorityArgsV2 = {\n  authority?: PluginAuthority;\n};`,
    '',
    `export type AssetAddablePluginArgsV2 =\n  | OwnerManagedPluginArgsV2\n  | AuthorityManagedPluginArgsV2;`,
    `export type AssetAllPluginArgsV2 =\n  | AssetAddablePluginArgsV2\n  | CreateOnlyPluginArgsV2;`,
    `export type AssetPluginAuthorityPairArgsV2 = AssetAllPluginArgsV2 &\n  AuthorityArgsV2;`,
    `export type AssetAddablePluginAuthorityPairArgsV2 = AssetAddablePluginArgsV2 &\n  AuthorityArgsV2;`,
    '',
    `export type CollectionAddablePluginArgsV2 = AuthorityManagedPluginArgsV2;`,
    `export type CollectionAllPluginArgsV2 =\n  | CreateOnlyPluginArgsV2\n  | CollectionAddablePluginArgsV2;`,
    `export type CollectionPluginAuthorityPairArgsV2 = CollectionAllPluginArgsV2 &\n  AuthorityArgsV2;`,
    `export type CollectionAddablePluginAuthorityPairArgsV2 =\n  CollectionAddablePluginArgsV2 & AuthorityArgsV2;`,
    '',
    pluginAliases,
    '',
    `export type CommonPluginsList = {\n${commonList.map(listEntry).join('\n')}\n};`,
    '',
    `export type AssetPluginsList = {\n${assetList
      .map(listEntry)
      .join('\n')}\n} & CommonPluginsList;`,
    '',
    `export type CollectionPluginsList = {\n${collectionList
      .map(listEntry)
      .join('\n')}\n} & CommonPluginsList;`,
    '',
    `export type PluginsList = AssetPluginsList & CollectionPluginsList;`,
    '',
    `export type GroupPluginsList = {\n${groupList.map(listEntry).join('\n')}\n};`,
    '',
  ].join('\n');

  return body;
}

function buildRenderMap(root, manifest) {
  const plugins = buildPlugins(root, manifest);
  const imp = makeImports(IMPORT_ORDER);
  const body = generateInternalFile(plugins, imp);
  const code = [HEADER, imp.render(), '', body].join('\n');
  return { 'internal.ts': code };
}

function renderInternalPlugins(root, jsGeneratedDir, prettierConfig, manifestPath) {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  return writeRenderMap(
    buildRenderMap(root, manifest),
    path.join(jsGeneratedDir, 'plugins'),
    prettierConfig
  ).map((f) => path.join('plugins', f));
}

module.exports = { buildRenderMap, buildPlugins, renderInternalPlugins };
