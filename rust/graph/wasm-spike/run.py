"""Reproduce the spike using the pinned Rust/C dependencies and wasm-bindgen CLI."""
from pathlib import Path
import argparse
import gzip
import json
import os
import subprocess
import sys

root = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--wasm-bindgen', default='wasm-bindgen')
parser.add_argument('--matrix', action='store_true', help='also measure each omitted grammar')
args = parser.parse_args()

def run(command, **kwargs):
    return subprocess.run(command, cwd=root, check=True, **kwargs)

run([sys.executable, 'prepare.py'])
run([sys.executable, 'corpus.py'])
metadata = json.loads(run(['cargo', 'metadata', '--format-version=1', '--locked', '--offline'], capture_output=True, text=True).stdout)
package = next(package for package in metadata['packages'] if package['name'] == 'tree-sitter-language')
headers = Path(package['manifest_path']).parent / 'wasm/include'
env = dict(os.environ, CC_wasm32_unknown_unknown='clang', AR_wasm32_unknown_unknown='llvm-ar',
           CFLAGS_wasm32_unknown_unknown=f'-I{headers} -include {root / "include/compat.h"}')
run(['cargo', 'build', '--release', '--locked', '--offline'])

sizes = {}
variants = ['all'] + (['python', 'r', 'javascript', 'typescript', 'julia', 'matlab', 'rust', 'snakemake'] if args.matrix else [])
try:
    for variant in variants:
        run([sys.executable, 'prepare.py'] + ([] if variant == 'all' else ['--omit', variant]))
        run(['cargo', 'build', '--lib', '--target=wasm32-unknown-unknown', '--release', '--locked', '--offline'], env=env)
        run([args.wasm_bindgen, '--target=nodejs', '--out-dir', f'pkg/{variant}', 'target/wasm32-unknown-unknown/release/stencila_code_analysis_spike.wasm'])
        (root / f'pkg/{variant}/package.json').write_text('{"type":"commonjs"}\n')
        raw = (root / f'pkg/{variant}/stencila_code_analysis_spike_bg.wasm').read_bytes()
        sizes[variant] = {'rawBytes':len(raw), 'gzipBytes':len(gzip.compress(raw, compresslevel=9, mtime=0))}
finally:
    # Restore the full analyzer even if a matrix build failed.
    run([sys.executable, 'prepare.py'])
(root / 'sizes.json').write_text(json.dumps(sizes, indent=2) + '\n')
run(['node', 'benchmark.cjs'])
