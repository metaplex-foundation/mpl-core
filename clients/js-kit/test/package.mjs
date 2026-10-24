import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, posix } from 'node:path';
import { fileURLToPath } from 'node:url';

// Install the actual tarball into an unrelated consumer; source-relative imports
// cannot hide missing exports, runtime dependencies, or declaration files here.
const packageDir = fileURLToPath(new URL('..', import.meta.url));
const consumer = mkdtempSync(join(tmpdir(), 'mpl-core-kit-consumer-'));
const run = (args, cwd = consumer) =>
  execFileSync('pnpm', args, { cwd, stdio: 'inherit' });
run(['pack', '--pack-destination', consumer], packageDir);
const archive = readdirSync(consumer).find((name) => name.endsWith('.tgz'));
assert.ok(archive, 'pnpm pack must produce a tarball');
const contents = execFileSync('tar', ['-tzf', join(consumer, archive)], {
  encoding: 'utf8',
})
  .trim()
  .split('\n')
  .map((path) => posix.normalize(path));
assert.ok(
  contents.includes('package/README.md'),
  'Package must include usage documentation'
);
assert.ok(
  contents.every((path) =>
    /^package\/(?:dist\/(?:generated|hooked)\/|dist\/index\.(?:js|d\.ts)$|package\.json$|README\.md$)/.test(
      path
    )
  ),
  'Package contains files outside the documented runtime distribution'
);

writeFileSync(
  join(consumer, 'package.json'),
  JSON.stringify({
    dependencies: {
      '@metaplex-foundation/mpl-core-kit': `file:${join(consumer, archive)}`,
      '@solana/kit': '8.5.0',
    },
    devDependencies: { '@types/node': '^24.0.3', typescript: '5.9.3' },
    private: true,
    type: 'module',
  })
);
const source = `
import {
  getAssetV1Codec,
  getCreateV2Instruction,
  PluginType,
  type AssetV1,
} from '@metaplex-foundation/mpl-core-kit';
import { address, createNoopSigner } from '@solana/kit';

const owner = address('11111111111111111111111111111111');
const instruction = getCreateV2Instruction({
  asset: createNoopSigner(owner),
  name: 'Consumer',
  payer: createNoopSigner(owner),
  uri: '',
});
if (instruction.data[0] !== 20) throw new Error('Wrong createV2 discriminator');
const codec = getAssetV1Codec();
const decoded: AssetV1 = codec.decode(codec.encode({
  name: 'Consumer',
  owner,
  plugins: [{
    authority: { __kind: 'Owner' },
    offset: 0n,
    plugin: { __kind: 'FreezeDelegate', fields: [{ frozen: true }] },
    pluginType: PluginType.FreezeDelegate,
  }],
  seq: null,
  updateAuthority: { __kind: 'Address', fields: [owner] },
  uri: '',
}));
if (decoded.plugins[0].plugin.__kind !== 'FreezeDelegate') {
  throw new Error('Consumer did not receive complete account data');
}
`;
writeFileSync(join(consumer, 'index.ts'), source);
run(['install', '--ignore-scripts']);
run([
  'exec',
  'tsc',
  '--module',
  'NodeNext',
  '--outDir',
  'dist',
  '--strict',
  '--target',
  'ES2022',
  'index.ts',
]);
execFileSync(process.execPath, ['dist/index.js'], {
  cwd: consumer,
  stdio: 'inherit',
});
console.log('Packed-package runtime and declaration checks passed.');
