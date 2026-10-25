/**
 * Custom Kinobi renderer for the mpl-core "external plugin adapter" wrapper layer.
 *
 * The Umi JavaScript renderer emits wire-shaped types (`__kind` discriminators,
 * `fields` tuples, `Option<T>`). The SDK exposes an ergonomic layer on top of
 * those (`type` discriminators, unwrapped options, flattened structs) plus a
 * `fromBase`/`initToBase`/`updateToBase`/manifest set per adapter. That layer
 * used to be hand-written, one ~90-line file per adapter, and every new adapter
 * required coordinated edits across ~6 files.
 *
 * This renderer derives the entire layer from the IDL node tree:
 *   - the adapter set comes from the `externalPluginAdapter` enum,
 *   - per-field transforms come from inspecting each `base*` defined type,
 *   - a small OVERRIDES table below captures the few things the IDL cannot
 *     express (which adapters carry inline `data`, redundant plugin-key fields,
 *     the DataSection `dataAuthority` derivation, and DataSection being
 *     non-updatable).
 *
 * Adding a "regular" external plugin adapter (like AgentIdentity) now needs
 * nothing here at all — add the Rust type and re-run `pnpm generate`.
 */
const path = require("path");
const { pascalCase, camelCase } = require("@metaplex-foundation/kinobi");
const { makeImports, header, writeRenderMap } = require("./kinobiRenderUtils.cjs");

const HEADER = header("kinobiExternalPluginAdapters.cjs");

// Import ordering for generated adapter files (anything else is appended, sorted).
const IMPORT_ORDER = [
  "@metaplex-foundation/umi",
  "..",
  "./base",
  "../../plugins/externalPluginAdapterManifest",
  "../../plugins/externalPluginAdapterKey",
  "../../plugins/lib",
  "../../plugins/pluginAuthority",
  "../../plugins/extraAccount",
  "../../plugins/lifecycleChecks",
  "../../plugins/validationResultsOffset",
  "../../plugins/linkedDataKey",
  "../../plugins/types",
];

// ---------------------------------------------------------------------------
// Substitution registry: base defined-type link name -> ergonomic replacement.
// These map a generated `Base*` leaf type to its hand-written ergonomic
// counterpart and the transform function pair that converts between them.
// ---------------------------------------------------------------------------
const sub = (ts, mod) => ({
  ts,
  from: `${camelCase(ts)}FromBase`,
  to: `${camelCase(ts)}ToBase`,
  mod: `../../plugins/${mod}`,
});

const SUBS = {
  basePluginAuthority: sub("PluginAuthority", "pluginAuthority"),
  baseExtraAccount: sub("ExtraAccount", "extraAccount"),
  baseValidationResultsOffset: sub(
    "ValidationResultsOffset",
    "validationResultsOffset"
  ),
  baseLinkedDataKey: sub("LinkedDataKey", "linkedDataKey"),
};

const LIFECYCLE = sub("LifecycleChecks", "lifecycleChecks");

// ---------------------------------------------------------------------------
// Per-adapter overrides for things the IDL does not encode.
// Keyed by PascalCase adapter name. Every field is optional; a brand-new
// regular adapter needs no entry at all. Code snippets declare the imports
// they need as `{ code, imports: [[module, ...names]] }`.
// ---------------------------------------------------------------------------
const PUBLIC_KEY = ["@metaplex-foundation/umi", "PublicKey"];
const PLUGIN_AUTHORITY = [SUBS.basePluginAuthority.mod, "PluginAuthority"];
const LIFECYCLE_CHECKS = [LIFECYCLE.mod, "LifecycleChecks"];

const HOOKED_PROGRAM_KEY = {
  code: "hookedProgram: PublicKey;",
  imports: [PUBLIC_KEY],
};
const DATA_AUTHORITY_KEY = {
  code: "dataAuthority: PluginAuthority;",
  imports: [PLUGIN_AUTHORITY],
};
// Preserve the historically-present (record-level) lifecycle checks field on
// the ergonomic init args even though it is absent from the base type.
const RECORD_LIFECYCLE_CHECKS = {
  code: "lifecycleChecks?: LifecycleChecks;",
  imports: [LIFECYCLE_CHECKS],
};

const OVERRIDES = {
  LifecycleHook: {
    hasDataField: true,
    injectData: true,
    pluginKeyExtra: HOOKED_PROGRAM_KEY,
  },
  AppData: {
    hasDataField: true,
    injectData: true,
    pluginKeyExtra: DATA_AUTHORITY_KEY,
    extraInitFields: RECORD_LIFECYCLE_CHECKS,
    extraInitOmit: ["lifecycleChecks"],
  },
  LinkedLifecycleHook: {
    hasDataField: true,
    pluginKeyExtra: HOOKED_PROGRAM_KEY,
  },
  LinkedAppData: {
    hasDataField: true,
    pluginKeyExtra: DATA_AUTHORITY_KEY,
    extraInitFields: RECORD_LIFECYCLE_CHECKS,
    extraInitOmit: ["lifecycleChecks"],
  },
  DataSection: {
    hasDataField: true,
    injectData: true,
    // dataAuthority is not a stored field; it is derived from parentKey.
    extraTypeFields: {
      code: "dataAuthority?: PluginAuthority;",
      imports: [PLUGIN_AUTHORITY],
    },
    extraTypeOmit: ["dataAuthority"],
    extraFromBase: {
      code: "dataAuthority: input.parentKey.__kind !== 'LinkedLifecycleHook' ? pluginAuthorityFromBase(input.parentKey.fields[0]) : undefined,",
      imports: [[SUBS.basePluginAuthority.mod, SUBS.basePluginAuthority.from]],
    },
    updatable: false,
  },
  // The adapters-list key defaults to `${camelCase(name)}s`.
  AgentIdentity: { listKey: "agentIdentities" },
};

// ---------------------------------------------------------------------------
// Node helpers.
// ---------------------------------------------------------------------------
function tsPlain(node) {
  switch (node.kind) {
    case "stringTypeNode":
      return { ts: "string" };
    case "publicKeyTypeNode":
      return { ts: "PublicKey", umi: "PublicKey" };
    case "booleanTypeNode":
      return { ts: "boolean" };
    case "bytesTypeNode":
      return { ts: "Uint8Array" };
    case "numberTypeNode":
      return {
        ts: /64|128/.test(node.format || "") ? "number | bigint" : "number",
      };
    default:
      throw new Error(`tsPlain: unsupported passthrough node kind "${node.kind}"`);
  }
}

function isLifecycleChecks(node) {
  return (
    node.kind === "arrayTypeNode" &&
    node.item.kind === "tupleTypeNode" &&
    node.item.items.length === 2 &&
    node.item.items[0].kind === "definedTypeLinkNode" &&
    String(node.item.items[0].name) === "hookableLifecycleEvent"
  );
}

/**
 * Classify a struct field node into one of the transform categories.
 */
function classify(field) {
  const name = String(field.name);
  let t = field.type;
  let optional = false;
  if (t.kind === "optionTypeNode") {
    optional = true;
    t = t.item;
  }
  if (isLifecycleChecks(t)) {
    return { name, optional, cat: "lifecycle" };
  }
  if (t.kind === "arrayTypeNode" && t.item.kind === "definedTypeLinkNode") {
    const s = SUBS[String(t.item.name)];
    if (s) return { name, optional, cat: "subArray", sub: s };
  }
  if (t.kind === "definedTypeLinkNode") {
    const linkName = String(t.name);
    const s = SUBS[linkName];
    if (s) return { name, optional, cat: "sub", sub: s };
    if (linkName === "externalPluginAdapterSchema") {
      return { name, optional, cat: "schema" };
    }
    return { name, optional, cat: "passthrough", node: t };
  }
  return { name, optional, cat: "passthrough", node: t };
}

// True when a field needs an ergonomic override (i.e. is not a plain
// pass-through of a required leaf type).
function isOverridden(c) {
  if (c.cat === "sub" || c.cat === "subArray" || c.cat === "lifecycle")
    return true;
  return c.optional;
}

// ---------------------------------------------------------------------------
// Per-adapter file generation.
// ---------------------------------------------------------------------------
function generateAdapterFile(adapter) {
  const { pascal, camel, ov, baseFields, initFields, updateFields } = adapter;
  const imp = makeImports(IMPORT_ORDER);
  imp.use("..", `Base${pascal}`, "ExternalRegistryRecord");
  imp.use(
    "../../plugins/externalPluginAdapterManifest",
    "ExternalPluginAdapterManifest"
  );
  imp.use("./base", "BaseExternalPluginAdapter");

  const useSubType = (s) => imp.use(s.mod, s.ts);
  const useSubFrom = (s) => imp.use(s.mod, s.from);
  const useSubTo = (s) => imp.use(s.mod, s.to);

  // ---- ergonomic data type ----
  const dataOmit = [];
  const dataOverrides = [];
  baseFields.forEach((c) => {
    if (c.cat === "sub" || c.cat === "subArray") {
      dataOmit.push(c.name);
      const q = c.optional ? "?" : "";
      const ts = c.cat === "subArray" ? `Array<${c.sub.ts}>` : c.sub.ts;
      dataOverrides.push(`${c.name}${q}: ${ts};`);
      useSubType(c.sub);
    }
  });
  (ov.extraTypeOmit || []).forEach((n) => dataOmit.push(n));
  if (ov.extraTypeFields) dataOverrides.push(imp.snippet(ov.extraTypeFields));
  if (ov.hasDataField) dataOverrides.push("data?: any;");

  let dataType;
  if (dataOmit.length === 0 && dataOverrides.length === 0) {
    dataType = `export type ${pascal} = Base${pascal};`;
  } else {
    const omit =
      dataOmit.length > 0
        ? `Omit<Base${pascal}, ${dataOmit.map((n) => `'${n}'`).join(" | ")}>`
        : `Base${pascal}`;
    dataType = `export type ${pascal} = ${omit} & {\n  ${dataOverrides.join(
      "\n  "
    )}\n};`;
  }

  // ---- plugin type ----
  let pluginType = `export type ${pascal}Plugin = BaseExternalPluginAdapter &\n  ${pascal} & {\n    type: '${pascal}';`;
  if (ov.pluginKeyExtra) pluginType += `\n    ${imp.snippet(ov.pluginKeyExtra)}`;
  pluginType += "\n  };";

  // ---- init args type ----
  const buildArgs = (label, fields, extraOmit, extraFields, discriminantKey) => {
    const omit = [];
    const readd = [];
    fields.forEach((c) => {
      if (!isOverridden(c)) return;
      omit.push(c.name);
      const q = c.optional ? "?" : "";
      if (c.cat === "lifecycle") {
        readd.push(`${c.name}${q}: LifecycleChecks;`);
        useSubType(LIFECYCLE);
      } else if (c.cat === "subArray") {
        readd.push(`${c.name}${q}: Array<${c.sub.ts}>;`);
        useSubType(c.sub);
      } else if (c.cat === "sub") {
        readd.push(`${c.name}${q}: ${c.sub.ts};`);
        useSubType(c.sub);
      } else if (c.cat === "schema") {
        readd.push(`${c.name}?: ExternalPluginAdapterSchema;`);
        imp.use("..", "ExternalPluginAdapterSchema");
      } else if (c.cat === "passthrough") {
        const p = tsPlain(c.node);
        readd.push(`${c.name}?: ${p.ts};`);
        if (p.umi) imp.use("@metaplex-foundation/umi", p.umi);
      }
    });
    (extraOmit || []).forEach((n) => omit.push(n));
    const readdExtra = [];
    if (discriminantKey === "type") {
      readdExtra.push(`type: '${pascal}';`);
    } else {
      readdExtra.push("key: ExternalPluginAdapterKey;");
      imp.use("../../plugins/externalPluginAdapterKey", "ExternalPluginAdapterKey");
    }
    if (extraFields) readdExtra.push(imp.snippet(extraFields));
    const baseArgs = `Base${pascal}${label}Args`;
    imp.use("..", baseArgs);
    const body = [...readdExtra, ...readd].join("\n  ");
    const lhs =
      omit.length > 0
        ? `Omit<${baseArgs}, ${omit.map((n) => `'${n}'`).join(" | ")}>`
        : baseArgs;
    return `export type ${pascal}${label}Args = ${lhs} & {\n  ${body}\n};`;
  };

  const initType = buildArgs(
    "InitInfo",
    initFields,
    ov.extraInitOmit,
    ov.extraInitFields,
    "type"
  );
  const updateType = buildArgs(
    "UpdateInfo",
    updateFields,
    ov.extraUpdateOmit,
    ov.extraUpdateFields,
    "key"
  );

  // ---- toBase functions ----
  const buildToBase = (label, fields) => {
    const baseArgs = `Base${pascal}${label}Args`;
    const lines = fields.map((c) => {
      const s = c.cat === "lifecycle" ? LIFECYCLE : c.sub;
      if (s) {
        useSubTo(s);
        const conv =
          c.cat === "subArray"
            ? `input.${c.name}.map(${s.to})`
            : `${s.to}(input.${c.name})`;
        return c.optional
          ? `${c.name}: input.${c.name} ? ${conv} : null,`
          : `${c.name}: ${conv},`;
      }
      // schema / passthrough
      return c.optional
        ? `${c.name}: input.${c.name} ?? null,`
        : `${c.name}: input.${c.name},`;
    });
    const fnName = `${camel}${label}ArgsToBase`;
    const body =
      lines.length > 0 ? `return {\n    ${lines.join("\n    ")}\n  };` : "return {};";
    return `export function ${fnName}(\n  input: ${pascal}${label}Args\n): ${baseArgs} {\n  ${body}\n}`;
  };

  const initToBase = buildToBase("InitInfo", initFields);
  const updateToBase = buildToBase("UpdateInfo", updateFields);

  // ---- fromBase function ----
  const fromLines = [];
  baseFields.forEach((c) => {
    if (c.cat !== "sub" && c.cat !== "subArray") return;
    useSubFrom(c.sub);
    const conv = (v) =>
      c.cat === "subArray" ? `${v}.map(${c.sub.from})` : `${c.sub.from}(${v})`;
    fromLines.push(
      c.optional
        ? `${c.name}: input.${c.name}.__option === 'Some' ? ${conv(
            `input.${c.name}.value`
          )} : undefined,`
        : `${c.name}: ${conv(`input.${c.name}`)},`
    );
  });
  if (ov.extraFromBase) fromLines.push(imp.snippet(ov.extraFromBase));
  if (ov.injectData) {
    imp.use("../../plugins/lib", "parseExternalPluginAdapterData");
    fromLines.push("data: parseExternalPluginAdapterData(input, record, account),");
  }
  const fromBody =
    fromLines.length > 0
      ? `return {\n    ...input,\n    ${fromLines.join("\n    ")}\n  };`
      : "return { ...input };";
  const fromBase = `export function ${camel}FromBase(\n  input: Base${pascal},\n  record: ExternalRegistryRecord,\n  account: Uint8Array\n): ${pascal} {\n  ${fromBody}\n}`;

  // ---- manifest ----
  imp.use("..", `Base${pascal}InitInfoArgs`, `Base${pascal}UpdateInfoArgs`);
  const manifest = `export const ${camel}Manifest: ExternalPluginAdapterManifest<
  ${pascal},
  Base${pascal},
  ${pascal}InitInfoArgs,
  Base${pascal}InitInfoArgs,
  ${pascal}UpdateInfoArgs,
  Base${pascal}UpdateInfoArgs
> = {
  type: '${pascal}',
  fromBase: ${camel}FromBase,
  initToBase: ${camel}InitInfoArgsToBase,
  updateToBase: ${camel}UpdateInfoArgsToBase,
};`;

  return [
    HEADER,
    imp.render(),
    "",
    dataType,
    "",
    pluginType,
    "",
    initType,
    "",
    updateType,
    "",
    initToBase,
    "",
    updateToBase,
    "",
    fromBase,
    "",
    manifest,
    "",
  ].join("\n");
}

// ---------------------------------------------------------------------------
// base.ts (shared adapter types, kept value-free to avoid import cycles).
// ---------------------------------------------------------------------------
function generateBaseFile() {
  return `${HEADER}
import { BasePlugin } from '../../plugins/types';
import { LifecycleChecksContainer } from '../../plugins/lifecycleChecks';

export type ExternalPluginAdapterData = {
  dataLen?: bigint;
  dataOffset?: bigint;
};

export type BaseExternalPluginAdapter = BasePlugin &
  ExternalPluginAdapterData &
  LifecycleChecksContainer;
`;
}

// ---------------------------------------------------------------------------
// registry.ts (unions, manifests map, dispatch, init/update helpers).
// ---------------------------------------------------------------------------
function generateRegistryFile(adapters) {
  const imp = makeImports(IMPORT_ORDER);
  imp.use("@metaplex-foundation/umi", "isSome");
  imp.use(
    "..",
    "ExternalRegistryRecord",
    "getExternalPluginAdapterSerializer",
    "BaseExternalPluginAdapterInitInfoArgs",
    "BaseExternalPluginAdapterKey",
    "BaseExternalPluginAdapterUpdateInfoArgs"
  );
  imp.use("./base", "BaseExternalPluginAdapter", "ExternalPluginAdapterData");
  imp.use(SUBS.basePluginAuthority.mod, SUBS.basePluginAuthority.from);
  imp.use(LIFECYCLE.mod, LIFECYCLE.from);

  adapters.forEach((a) => {
    imp.use(
      `./${a.camel}`,
      `${a.pascal}Plugin`,
      `${a.pascal}InitInfoArgs`,
      `${a.camel}Manifest`
    );
    if (a.updatable !== false) imp.use(`./${a.camel}`, `${a.pascal}UpdateInfoArgs`);
  });

  const typeString = `export type ExternalPluginAdapterTypeString =\n  BaseExternalPluginAdapterKey['__kind'];`;

  const dataReexport = `export type { BaseExternalPluginAdapter, ExternalPluginAdapterData };`;

  const unionAdapters = `export type ExternalPluginAdapters =\n  | ${adapters
    .map((a) => `${a.pascal}Plugin`)
    .join("\n  | ")};`;

  const listType = `export type ExternalPluginAdaptersList = {\n${adapters
    .map((a) => `  ${a.listKey}?: ${a.pascal}Plugin[];`)
    .join("\n")}\n};`;

  const argsUnion = (name, label, members) =>
    `export type ${name} =\n  | ${members
      .map((a) => `({\n      type: '${a.pascal}';\n    } & ${a.pascal}${label}Args)`)
      .join("\n  | ")};`;
  const initUnion = argsUnion(
    "ExternalPluginAdapterInitInfoArgs",
    "InitInfo",
    adapters
  );
  const updateUnion = argsUnion(
    "ExternalPluginAdapterUpdateInfoArgs",
    "UpdateInfo",
    adapters.filter((a) => a.updatable !== false)
  );

  const manifests = `export const externalPluginAdapterManifests = {\n${adapters
    .map((a) => `  ${a.pascal}: ${a.camel}Manifest,`)
    .join("\n")}\n};`;

  const meta = `const externalPluginAdapterMeta: Record<\n  ExternalPluginAdapterTypeString,\n  { listKey: keyof ExternalPluginAdaptersList; dataStore: boolean }\n> = {\n${adapters
    .map(
      (a) =>
        `  ${a.pascal}: { listKey: '${a.listKey}', dataStore: ${Boolean(
          a.injectData
        )} },`
    )
    .join("\n")}\n};`;

  const isType = `export const isExternalPluginAdapterType = (plugin: { type: string }) =>\n  plugin.type in externalPluginAdapterManifests;`;

  const createInit = `export function createExternalPluginAdapterInitInfo({\n  type,\n  ...args\n}: ExternalPluginAdapterInitInfoArgs): BaseExternalPluginAdapterInitInfoArgs {\n  const manifest = externalPluginAdapterManifests[type];\n  return {\n    __kind: type,\n    fields: [manifest.initToBase(args as any)] as any,\n  };\n}`;

  const createUpdate = `export function createExternalPluginAdapterUpdateInfo({\n  type,\n  ...args\n}: ExternalPluginAdapterUpdateInfoArgs): BaseExternalPluginAdapterUpdateInfoArgs {\n  const manifest = externalPluginAdapterManifests[type];\n  return {\n    __kind: type,\n    fields: [manifest.updateToBase(args as any)] as any,\n  };\n}`;

  const dispatch = `export function externalRegistryRecordsToExternalPluginAdapterList(
  records: ExternalRegistryRecord[],
  accountData: Uint8Array
): ExternalPluginAdaptersList {
  const result: ExternalPluginAdaptersList = {};

  records.forEach((record) => {
    const deserializedPlugin = getExternalPluginAdapterSerializer().deserialize(
      accountData,
      Number(record.offset)
    )[0];

    const base: BaseExternalPluginAdapter = {
      lifecycleChecks:
        record.lifecycleChecks.__option === 'Some'
          ? lifecycleChecksFromBase(record.lifecycleChecks.value)
          : undefined,
      authority: pluginAuthorityFromBase(record.authority),
      offset: record.offset,
    };

    const type = deserializedPlugin.__kind as ExternalPluginAdapterTypeString;
    const meta = externalPluginAdapterMeta[type];
    if (!meta) return;

    const dataFields: ExternalPluginAdapterData = meta.dataStore
      ? {
          dataOffset: isSome(record.dataOffset)
            ? record.dataOffset.value
            : undefined,
          dataLen: isSome(record.dataLen) ? record.dataLen.value : undefined,
        }
      : {};

    const manifest = externalPluginAdapterManifests[type];
    const list = (result[meta.listKey] ??= [] as any) as any[];
    list.push({
      type,
      ...dataFields,
      ...base,
      ...manifest.fromBase(deserializedPlugin.fields[0] as any, record, accountData),
    });
  });

  return result;
}`;

  return [
    HEADER,
    imp.render(),
    "",
    typeString,
    "",
    dataReexport,
    "",
    unionAdapters,
    "",
    listType,
    "",
    initUnion,
    "",
    updateUnion,
    "",
    manifests,
    "",
    meta,
    "",
    isType,
    "",
    createInit,
    "",
    createUpdate,
    "",
    dispatch,
    "",
  ].join("\n");
}

function generateIndexFile(adapters) {
  // `base` is intentionally omitted: its two types are re-exported through
  // `registry` (the ergonomic replacement for the old externalPluginAdapters
  // module), so listing it here too would be a duplicate star-export.
  const files = [...adapters.map((a) => a.camel), "registry"];
  return `${HEADER}\n${files.map((f) => `export * from './${f}';`).join("\n")}\n`;
}

// ---------------------------------------------------------------------------
// Entry point: build the render map from the root node.
// ---------------------------------------------------------------------------
/** Names of the `externalPluginAdapter` enum variants, in IDL order. */
function getExternalPluginAdapterNames(root) {
  const enumNode = root.programs[0].definedTypes.find(
    (dt) => String(dt.name) === "externalPluginAdapter"
  );
  if (!enumNode || enumNode.type.kind !== "enumTypeNode") {
    throw new Error("Could not find `externalPluginAdapter` enum in the IDL.");
  }
  return enumNode.type.variants.map((v) => String(v.name));
}

function buildAdapters(root) {
  const dtByName = new Map(
    root.programs[0].definedTypes.map((dt) => [String(dt.name), dt])
  );
  const fieldsOf = (name) => {
    const node = dtByName.get(name);
    if (!node || node.type.kind !== "structTypeNode") return [];
    return node.type.fields.map(classify);
  };

  return getExternalPluginAdapterNames(root).map((name) => {
    const pascal = pascalCase(name);
    const camel = camelCase(name);
    const baseName = `base${pascal}`;
    const ov = OVERRIDES[pascal] || {};

    return {
      pascal,
      camel,
      baseName,
      ov,
      listKey: ov.listKey || `${camel}s`,
      injectData: Boolean(ov.injectData),
      updatable: ov.updatable,
      baseFields: fieldsOf(baseName),
      initFields: fieldsOf(`${baseName}InitInfo`),
      updateFields: fieldsOf(`${baseName}UpdateInfo`),
    };
  });
}

function buildRenderMap(root) {
  const adapters = buildAdapters(root);
  const files = { "base.ts": generateBaseFile() };
  adapters.forEach((a) => {
    files[`${a.camel}.ts`] = generateAdapterFile(a);
  });
  files["registry.ts"] = generateRegistryFile(adapters);
  files["index.ts"] = generateIndexFile(adapters);
  return files;
}

function renderExternalPluginAdapters(root, jsGeneratedDir, prettierConfig) {
  return writeRenderMap(
    buildRenderMap(root),
    path.join(jsGeneratedDir, "plugins"),
    prettierConfig
  ).map((f) => path.join("plugins", f));
}

module.exports = {
  buildRenderMap,
  buildAdapters,
  getExternalPluginAdapterNames,
  renderExternalPluginAdapters,
};
