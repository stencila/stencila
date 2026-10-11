#!/usr/bin/env bash
set -euo pipefail

crate_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo_dir="$(cd "$crate_dir/../.." && pwd)"
harness="$crate_dir/tests/wasm"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_dir/target}"
export CARGO_TARGET_DIR="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$CARGO_TARGET_DIR")"

bindgen="${WASM_BINDGEN:-wasm-bindgen}"
if [[ "$("$bindgen" --version)" != "wasm-bindgen 0.2.121" ]]; then
    echo 'This smoke harness requires wasm-bindgen CLI 0.2.121' >&2
    exit 1
fi
eval "$(python3 "$crate_dir/scripts/wasm-env.py" --manifest-path "$harness/Cargo.toml")"
cargo check --locked --manifest-path "$crate_dir/Cargo.toml" -p stencila-code-analysis --target wasm32-unknown-unknown
cargo run --locked --quiet --manifest-path "$crate_dir/Cargo.toml" -p stencila-code-analysis --example corpus > "$harness/corpus.ndjson"
cargo build --locked --manifest-path "$harness/Cargo.toml" --release
cargo build --locked --manifest-path "$harness/Cargo.toml" --release --lib --target wasm32-unknown-unknown
"$bindgen" "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/code_analysis_wasm_smoke.wasm" --target nodejs --out-dir "$harness/pkg"
printf '{"type":"commonjs"}\n' > "$harness/pkg/package.json"
node "$harness/parity.cjs"
