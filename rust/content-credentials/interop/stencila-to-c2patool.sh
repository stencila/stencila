#!/usr/bin/env bash
set -euo pipefail

input=${1:-rust/content-credentials/tests/fixtures/sample.png}
out_dir=${2:-target/content-credentials-interop/stencila-to-c2patool}
c2patool_bin=${C2PATOOL_BIN:-c2patool}

input_name=$(basename "$input")
input_ext=${input_name##*.}
if [[ "$input_name" == "$input_ext" ]]; then
  signed_asset="$out_dir/stencila-signed"
else
  signed_asset="$out_dir/stencila-signed.$input_ext"
fi

run_stencila() {
  if [[ -n "${STENCILA_BIN:-}" ]]; then
    "$STENCILA_BIN" "$@"
  else
    cargo run --bin stencila -- "$@"
  fi
}

sha256_record() {
  local path=$1
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path"
  else
    shasum -a 256 "$path"
  fi
}

if ! command -v "$c2patool_bin" >/dev/null 2>&1; then
  echo "c2patool not found. Install it from https://opensource.contentauthenticity.org/docs/c2patool/" >&2
  exit 127
fi

mkdir -p "$out_dir"

{
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "stencila:"
  run_stencila --version || true
  echo
  echo "c2patool:"
  "$c2patool_bin" --version || true
} >"$out_dir/tool-versions.txt" 2>&1

run_stencila credentials init >"$out_dir/stencila-init.txt" 2>&1 || true

sign_args=(
  credentials sign
  "$input"
  --output "$signed_asset"
  --title "Stencila C2PA interop test"
)

if [[ -n "${STENCILA_CREDENTIALS_TSA_URL:-}" ]]; then
  sign_args+=(--tsa-url "$STENCILA_CREDENTIALS_TSA_URL")
fi

run_stencila "${sign_args[@]}" >"$out_dir/stencila-sign.txt" 2>&1

{
  echo "# input"
  sha256_record "$input"
  echo
  echo "# signed asset"
  sha256_record "$signed_asset"
  sidecar="${signed_asset%.*}.c2pa"
  if [[ -f "$sidecar" ]]; then
    echo
    echo "# sidecar"
    sha256_record "$sidecar"
  fi
} >"$out_dir/digests.txt"

run_stencila credentials verify "$signed_asset" >"$out_dir/stencila-verify.txt" 2>&1
run_stencila credentials verify "$signed_asset" --as json >"$out_dir/stencila-verify.json" 2>&1
run_stencila credentials inspect "$signed_asset" --as json >"$out_dir/stencila-inspect.json" 2>&1

"$c2patool_bin" "$signed_asset" >"$out_dir/c2patool-summary.json" 2>&1
"$c2patool_bin" "$signed_asset" --detailed >"$out_dir/c2patool-detailed.json" 2>&1

if [[ -n "${C2PATOOL_TRUST_ANCHORS:-}" ]]; then
  "$c2patool_bin" "$signed_asset" trust \
    --trust_anchors "$C2PATOOL_TRUST_ANCHORS" \
    >"$out_dir/c2patool-trust.json" 2>&1
fi

cat <<EOF
Interop evidence written to $out_dir

Signed asset:
  $signed_asset

Next manual check:
  Upload the signed asset to https://contentcredentials.org/verify
EOF
