const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {createHash} = require('node:crypto');
const { spawnSync } = require('node:child_process');
const { performance } = require('node:perf_hooks');
const root = __dirname;
const modulePath = path.join(root, 'pkg/all/stencila_code_analysis_spike.js');

function measureCase(input) {
  const api = require(modulePath);
  const initialBytes = api.linear_memory_bytes();
  const start = performance.now();
  const serialized = api.analyze_source(input.language, input.source);
  const coldMs = performance.now() - start;
  const output = JSON.parse(serialized);
  const samples = [];
  for (let i = 0; i < 5; i++) {
    const start = performance.now();
    const next = api.analyze_source(input.language, input.source);
    samples.push(performance.now() - start);
    assert.equal(next, serialized, input.path);
  }
  samples.sort((a,b) => a-b);
  return { language: input.language, kind: input.kind, path: input.path,
    lines: input.source.split('\n').length - 1,
    sourceBytes: Buffer.byteLength(input.source), coldMs, medianMs: samples[2],
    initialLinearMemoryBytes: initialBytes, peakLinearMemoryBytes: api.linear_memory_bytes(),
    peakProcessRssBytes: process.resourceUsage().maxRSS * 1024,
    syntaxError: output.syntax_error, ioFacts: output.io.length };
}

async function main() {
  const corpus = JSON.parse(fs.readFileSync(path.join(root, 'corpus.json'), 'utf8'));
  const fixtures = corpus.fixtures;
  const nativeInput = path.join(root, 'corpus.ndjson');
  fs.writeFileSync(nativeInput, fixtures.map(input => JSON.stringify(input)).join('\n') + '\n');
  const inputFd = fs.openSync(nativeInput, 'r');
  const native = spawnSync(path.join(root, 'target/release/stencila-code-analysis-spike'), [], {
    stdio: [inputFd, 'pipe', 'pipe'], timeout: 60000,
    encoding: 'utf8', maxBuffer: 64 * 1024 * 1024,
  });
  fs.closeSync(inputFd);
  if (native.error || native.status !== 0) throw new Error(native.stderr || String(native.error));
  const nativeFacts = native.stdout.trim().split('\n').map(line => JSON.parse(line));
  assert.equal(nativeFacts.length, fixtures.length);
  const api = require(modulePath);
  assert.throws(() => api.analyze_source('bash', 'cat data.csv'), error => error === 'Unsupported language: bash');
  const parity = fixtures.map((input, index) => {
    const facts = JSON.parse(api.analyze_source(input.language, input.source));
    assert.deepEqual(facts, nativeFacts[index], input.path);
    return {path: input.path, language: input.language, syntaxError: facts.syntax_error,
      factsSha256: createHash('sha256').update(JSON.stringify(facts)).digest('hex')};
  });
  const wasm = fs.readFileSync(path.join(root, 'pkg/all/stencila_code_analysis_spike_bg.wasm'));
  const imports = WebAssembly.Module.imports(new WebAssembly.Module(wasm));
  const startup = [];
  for (let i = 0; i < 20; i++) {
    const child = spawnSync(process.execPath, [__filename, '--startup'], {encoding:'utf8', timeout:60000});
    if (child.error || child.status !== 0) throw new Error(child.stderr || String(child.error));
    startup.push(JSON.parse(child.stdout));
  }
  const statistics = key => {
    const values = startup.map(sample => sample[key]).sort((a,b) => a-b);
    return {samples:20, medianMs:values[10], minMs:values[0], maxMs:values[19]};
  };
  const benchmarks = [];
  for (const [index, input] of corpus.benchmarks.entries()) {
    const child = spawnSync(process.execPath, [__filename, '--case', String(index)], {encoding:'utf8', timeout:60000});
    if (child.error || child.status !== 0) throw new Error(child.stderr || String(child.error));
    const result = JSON.parse(child.stdout);
    benchmarks.push(result);
    console.log(`${result.language} ${result.kind}: ${result.medianMs.toFixed(2)} ms; ${result.peakLinearMemoryBytes} bytes linear memory`);
  }
  const sizes = JSON.parse(fs.readFileSync(path.join(root, 'sizes.json'), 'utf8')).all;
  assert.equal(sizes.rawBytes, wasm.length);
  const output = {node: process.version, moduleSha256: createHash('sha256').update(wasm).digest('hex'), sizes,
    imports, startup:{compile:statistics('compileMs'), instantiate:statistics('instantiateMs'), load:statistics('loadMs')},
    parity, benchmarks};
  fs.writeFileSync(path.join(root, 'results.json'), JSON.stringify(output, null, 2) + '\n');
  console.log(`Parity passed for ${parity.length} sources`);
}
if (process.argv[2] === '--startup') {
  const start = performance.now();
  const bytes = fs.readFileSync(path.join(root, 'pkg/all/stencila_code_analysis_spike_bg.wasm'));
  const compileStart = performance.now();
  const module = new WebAssembly.Module(bytes);
  const compileMs = performance.now() - compileStart;
  const imports = {};
  for (const imported of WebAssembly.Module.imports(module)) {
    (imports[imported.module] ??= {})[imported.name] = () => {throw new Error('Unexpected host callback during startup');};
  }
  const instantiateStart = performance.now();
  new WebAssembly.Instance(module, imports);
  const instantiateMs = performance.now() - instantiateStart;
  console.log(JSON.stringify({compileMs, instantiateMs, loadMs:performance.now() - start}));
} else if (process.argv[2] === '--case') {
  const corpus = JSON.parse(fs.readFileSync(path.join(root, 'corpus.json'), 'utf8'));
  console.log(JSON.stringify(measureCase(corpus.benchmarks[Number(process.argv[3])])));
} else {
  main().catch(error => {console.error(error); process.exitCode=1;});
}
