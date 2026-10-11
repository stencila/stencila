const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawnSync } = require('node:child_process');
const zlib = require('node:zlib');
const crypto = require('node:crypto');
const wasm = require('./pkg/code_analysis_wasm_smoke.js');

const target = process.env.CARGO_TARGET_DIR;
assert.ok(target, 'run with the script that sets CARGO_TARGET_DIR');
const corpus = fs.readFileSync(path.join(__dirname, 'corpus.ndjson'), 'utf8').trim().split('\n');
assert.equal(corpus.length, 31);
assert.deepEqual(new Set(corpus.map(line => JSON.parse(line).language)), new Set([
  'python', 'r', 'javascript', 'typescript', 'julia', 'matlab', 'rust', 'snakemake', 'nextflow',
]));
assert.doesNotThrow(() => wasm.allocator_smoke(), 'linked allocator ABI edge cases');
assert.equal(wasm.detects('logical/Snakefile'), 'snakemake');
assert.equal(wasm.detects('logical/nextflow.config'), 'nextflow');
assert.equal(wasm.detects('unsupported.sh'), undefined);

// Repeated large parses must use a working allocator, not the scanner-only
// uninitialized, fixed-size bump heap. Compare each result to the native build.
const large = JSON.stringify({path: 'generated/python-5000.py', language: 'python',
  source: Array.from({length: 5000}, (_, index) => index % 25
    ? `value_${index} = ${index}` : `value_${index} = open("data/input.csv")`).join('\n') + '\n'});
const lines = [...corpus, ...Array(3).fill(large)];
const inputDirectory = fs.mkdtempSync(path.join(os.tmpdir(), 'code-analysis-parity-'));
const inputFile = path.join(inputDirectory, 'input.ndjson');
fs.writeFileSync(inputFile, lines.join('\n') + '\n');
const input = fs.openSync(inputFile, 'r');
let native;
try {
  native = spawnSync(path.join(target, 'release/code-analysis-wasm-smoke'), [], {
    stdio: [input, 'pipe', 'pipe'], encoding: 'utf8', maxBuffer: 64 * 1024 * 1024,
  });
} finally {
  fs.closeSync(input);
  fs.rmSync(inputDirectory, {recursive: true});
}
if (native.error) throw native.error;
assert.equal(native.status, 0, native.stderr);
const expected = native.stdout.trim().split('\n');
assert.equal(expected.length, lines.length);
for (let index = 0; index < lines.length; index++) {
  const actual = JSON.parse(wasm.analyze(lines[index]));
  assert.deepEqual(actual, JSON.parse(expected[index]), JSON.parse(lines[index]).path);
}
const bytes = fs.readFileSync(path.join(__dirname, 'pkg/code_analysis_wasm_smoke_bg.wasm'));
const moduleInfo = new WebAssembly.Module(bytes);
const exported = new Set(WebAssembly.Module.exports(moduleInfo).map(item => item.name));
for (const symbol of ['malloc', 'calloc', 'realloc', 'free', 'abort']) {
  assert.ok(exported.has(symbol), `missing C allocator symbol: ${symbol}`);
}
assert.ok(WebAssembly.Module.imports(moduleInfo).every(item => item.module === './code_analysis_wasm_smoke_bg.js' && item.kind === 'function' && item.name.startsWith('__wbindgen')), 'no host libc or WASI imports');
console.log(JSON.stringify({fixtures: corpus.length, allocatorStress: 3, bytes: bytes.length,
  gzipBytes: zlib.gzipSync(bytes, {level: 9}).length,
  sha256: crypto.createHash('sha256').update(bytes).digest('hex')}, null, 2));
