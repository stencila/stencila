#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <signed-asset> [output-dir]" >&2
  exit 64
fi

asset=$1
out_dir=${2:-target/content-credentials-interop/other-to-stencila}
c2patool_bin=${C2PATOOL_BIN:-c2patool}

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

if [[ ! -f "$asset" ]]; then
  echo "asset not found: $asset" >&2
  exit 66
fi

if ! command -v "$c2patool_bin" >/dev/null 2>&1; then
  echo "c2patool not found. Install it from https://opensource.contentauthenticity.org/docs/c2patool/" >&2
  exit 127
fi

mkdir -p "$out_dir"

sidecar=${C2PA_SIDECAR:-${asset%.*}.c2pa}
c2patool_sidecar_args=()
if [[ -f "$sidecar" ]]; then
  c2patool_sidecar_args=(--external-manifest "$sidecar")
fi

{
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "stencila:"
  run_stencila --version || true
  echo
  echo "c2patool:"
  "$c2patool_bin" --version || true
} >"$out_dir/tool-versions.txt" 2>&1

{
  echo "# asset"
  sha256_record "$asset"
  if [[ -f "$sidecar" ]]; then
    echo
    echo "# sidecar"
    sha256_record "$sidecar"
  fi
} >"$out_dir/digests.txt"

run_stencila credentials verify "$asset" >"$out_dir/stencila-verify.txt" 2>&1
run_stencila credentials verify "$asset" --as json >"$out_dir/stencila-verify.json" 2>&1
run_stencila credentials inspect "$asset" --as json >"$out_dir/stencila-inspect.json" 2>&1

"$c2patool_bin" "$asset" "${c2patool_sidecar_args[@]}" >"$out_dir/c2patool-summary.json" 2>&1
"$c2patool_bin" "$asset" --detailed "${c2patool_sidecar_args[@]}" >"$out_dir/c2patool-detailed.json" 2>&1

if [[ -n "${C2PATOOL_TRUST_ANCHORS:-}" ]]; then
  "$c2patool_bin" "$asset" "${c2patool_sidecar_args[@]}" trust \
    --trust_anchors "$C2PATOOL_TRUST_ANCHORS" \
    >"$out_dir/c2patool-trust.json" 2>&1
fi

cat <<EOF
Interop evidence written to $out_dir

Asset:
  $asset
EOF
