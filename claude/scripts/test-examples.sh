#!/bin/sh
# Golden test: every fenced `smd` block in the skills must parse.
#
# Extracts each fenced code block whose info string starts with `smd` from
# claude/skills/** (including generated references) into temporary files and
# converts each with `stencila convert --from smd --to json`, failing on any
# parse error.

set -e

cd "$(dirname "$0")/.."

STENCILA="${STENCILA:-}"
if [ -z "$STENCILA" ]; then
    if [ -x ../target/debug/stencila ]; then
        STENCILA=../target/debug/stencila
    elif command -v stencila > /dev/null 2>&1; then
        STENCILA=stencila
    else
        echo "test-examples.sh: no stencila CLI found (set STENCILA or install it)" >&2
        exit 1
    fi
fi

TMPDIR="${TMPDIR:-/tmp}"
WORKDIR=$(mktemp -d "$TMPDIR/stencila-skill-examples.XXXXXX")
trap 'rm -rf "$WORKDIR"' EXIT

# Extract every smd-fenced block into numbered files under $WORKDIR,
# with a sidecar .src file recording origin for error messages
find skills -name '*.md' | sort | while read -r file; do
    awk -v outdir="$WORKDIR" -v src="$file" '
        BEGIN { n = 0; safe = src; gsub(/\//, "_", safe) }
        # Opening fence: three or more backticks followed by smd (with
        # optional following keywords such as "demo")
        !infence && match($0, /^```+smd( |$)/) {
            fence = $0
            sub(/smd.*$/, "", fence)
            fencelen = length(fence)
            infence = 1
            n += 1
            outfile = outdir "/" safe "." n ".smd"
            printf "" > outfile
            next
        }
        # Closing fence: at least as many backticks, nothing else
        infence && $0 ~ /^```+[ \t]*$/ {
            close_fence = $0
            sub(/[ \t]*$/, "", close_fence)
            if (length(close_fence) >= fencelen) {
                infence = 0
                close(outfile)
                next
            }
        }
        infence { print > outfile }
    ' "$file"
done

count=0
failures=0
for example in "$WORKDIR"/*.smd; do
    [ -e "$example" ] || continue
    count=$((count + 1))
    if ! err=$("$STENCILA" convert "$example" --from smd --to json 2>&1 > /dev/null); then
        failures=$((failures + 1))
        echo "FAIL: $(basename "$example")" >&2
        echo "$err" | head -5 >&2
    fi
done

if [ "$count" -eq 0 ]; then
    echo "test-examples.sh: no smd examples found — extraction is broken" >&2
    exit 1
fi

if [ "$failures" -gt 0 ]; then
    echo "test-examples.sh: $failures of $count examples failed to parse" >&2
    exit 1
fi

echo "test-examples.sh: all $count smd examples parse"
