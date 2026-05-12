#!/usr/bin/env bash
set -uo pipefail

out_dir=${1:-target/content-credentials-interop/evidence}
other_asset=${2:-${THIRD_PARTY_ASSET:-}}
c2patool_bin=${C2PATOOL_BIN:-c2patool}

fixtures=(
  "png:rust/content-credentials/tests/fixtures/sample.png"
  "jpeg:rust/content-credentials/tests/fixtures/sample.jpg"
  "webp:rust/content-credentials/tests/fixtures/sample.webp"
  "svg:rust/content-credentials/tests/fixtures/sample.svg"
  "pdf:rust/content-credentials/tests/fixtures/sample.pdf"
)

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

record_result() {
  local name=$1
  local status=$2
  local log=$3

  if [[ "$status" -eq 0 ]]; then
    printf '| %s | pass | %s |\n' "$name" "$log" >>"$manifest"
  else
    printf '| %s | fail (%s) | %s |\n' "$name" "$status" "$log" >>"$manifest"
  fi
}

mkdir -p "$out_dir"
manifest="$out_dir/MANIFEST.txt"

{
  echo "# Stencila Content Credentials Evidence"
  echo
  echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "Output directory: $out_dir"
  echo
  echo "## Tool Versions"
  echo
  echo "Stencila:"
  run_stencila --version 2>&1 || true
  echo
  echo "c2patool:"
  if command -v "$c2patool_bin" >/dev/null 2>&1; then
    "$c2patool_bin" --version 2>&1 || true
  else
    echo "not found: $c2patool_bin"
  fi
  echo
  echo "## Signing Configuration"
  echo
  echo "Projection profile: public"
  if [[ -n "${STENCILA_CREDENTIALS_CERT:-}" || -n "${STENCILA_CREDENTIALS_KEY:-}" ]]; then
    echo "Signing identity: external certificate from STENCILA_CREDENTIALS_CERT/KEY"
  else
    echo "Signing identity: local signing identity from credentials init"
  fi
  if [[ -n "${STENCILA_CREDENTIALS_TSA_URL:-}" ]]; then
    echo "Timestamp authority: $STENCILA_CREDENTIALS_TSA_URL"
  else
    echo "Timestamp authority: none configured"
  fi
  echo
  echo "## Fixture Digests"
  echo
  for fixture in "${fixtures[@]}"; do
    format=${fixture%%:*}
    path=${fixture#*:}
    if [[ -f "$path" ]]; then
      printf '%s: ' "$format"
      sha256_record "$path"
    else
      echo "$format: missing $path"
    fi
  done
  if [[ -n "$other_asset" ]]; then
    echo "third-party: $(sha256_record "$other_asset" 2>/dev/null || echo "missing $other_asset")"
  fi
  echo
  echo "## Results"
  echo
  echo "| Test | Result | Evidence |"
  echo "| --- | --- | --- |"
} >"$manifest"

if ! command -v "$c2patool_bin" >/dev/null 2>&1; then
  record_result "setup" 127 "c2patool not found"
  echo "c2patool not found. Install it from https://opensource.contentauthenticity.org/docs/c2patool/" >&2
  echo "Evidence manifest written to $manifest"
  exit 127
fi

for fixture in "${fixtures[@]}"; do
  format=${fixture%%:*}
  path=${fixture#*:}
  evidence_dir="$out_dir/stencila-to-c2patool/$format"
  log="$evidence_dir/run.log"
  mkdir -p "$evidence_dir"

  if [[ ! -f "$path" ]]; then
    echo "fixture not found: $path" >"$log"
    record_result "Stencila -> c2patool ($format)" 66 "$log"
    continue
  fi

  rust/content-credentials/interop/stencila-to-c2patool.sh \
    "$path" \
    "$evidence_dir" \
    >"$log" 2>&1
  status=$?
  record_result "Stencila -> c2patool ($format)" "$status" "$evidence_dir"
done

if [[ -n "$other_asset" ]]; then
  evidence_dir="$out_dir/other-to-stencila"
  log="$evidence_dir/run.log"
  mkdir -p "$evidence_dir"

  if [[ ! -f "$other_asset" ]]; then
    echo "third-party asset not found: $other_asset" >"$log"
    record_result "Other -> Stencila" 66 "$log"
  else
    rust/content-credentials/interop/other-to-stencila.sh \
      "$other_asset" \
      "$evidence_dir" \
      >"$log" 2>&1
    status=$?
    record_result "Other -> Stencila" "$status" "$evidence_dir"
  fi
else
  printf '| %s | skipped | %s |\n' \
    "Other -> Stencila" \
    "pass a third-party signed asset as argument 2 or set THIRD_PARTY_ASSET" \
    >>"$manifest"
fi

cat <<EOF
Evidence written to $out_dir
Manifest:
  $manifest
EOF
