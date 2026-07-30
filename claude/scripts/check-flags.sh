#!/bin/sh
# Flag drift guard: every `--flag` mentioned in a skill must exist in the
# live `--help` output of a stencila command mentioned in the same file.
#
# For each skill file, collects the set of `stencila <cmd> [<subcmd>]`
# commands it mentions, concatenates their `--help` output, and checks every
# `--flag` token in the file against that text (plus top-level help).
# Flags after a ` -- ` separator are passthrough/document arguments, not CLI
# flags, and are ignored.

set -e

cd "$(dirname "$0")/.."

STENCILA="${STENCILA:-}"
if [ -z "$STENCILA" ]; then
    if [ -x ../target/debug/stencila ]; then
        STENCILA=../target/debug/stencila
    elif command -v stencila > /dev/null 2>&1; then
        STENCILA=stencila
    else
        echo "check-flags.sh: no stencila CLI found (set STENCILA or install it)" >&2
        exit 1
    fi
fi

TMPDIR="${TMPDIR:-/tmp}"
WORKDIR=$(mktemp -d "$TMPDIR/stencila-check-flags.XXXXXX")
trap 'rm -rf "$WORKDIR"' EXIT

export NO_COLOR=1

# Cached help output for a command ("" for top-level)
help_for() {
    cache="$WORKDIR/help.$(echo "$1" | tr ' /' '__')"
    if [ ! -f "$cache" ]; then
        # shellcheck disable=SC2086
        "$STENCILA" $1 --help > "$cache" 2> /dev/null || : > "$cache"
    fi
    printf '%s\n' "$cache"
}

failures=0

for file in $(find skills -name '*.md' | sort); do
    # Skip generated references: their prose is checked upstream in site/docs
    case "$file" in
        */references/*) continue ;;
    esac

    # Commands mentioned in this file, as one- or two-word forms; action
    # skills write invocations as `<stencila> <cmd>` placeholders
    cmds=$(grep -oE '(stencila|<stencila>) [a-z]+( [a-z]+)?' "$file" \
        | sed 's/^<stencila> //; s/^stencila //' | sort -u)

    # Concatenate help for each mentioned command (two-word form falling
    # back to one-word), plus top-level help
    : > "$WORKDIR/help.all"
    cat "$(help_for "")" >> "$WORKDIR/help.all"
    for cmd in $(echo "$cmds" | tr ' ' '+'); do
        two=$(echo "$cmd" | tr '+' ' ')
        one=${two%% *}
        h=$(help_for "$two")
        if [ ! -s "$h" ]; then
            h=$(help_for "$one")
        fi
        cat "$h" >> "$WORKDIR/help.all"
        cat "$(help_for "$one")" >> "$WORKDIR/help.all"
    done

    # Flags used in this file: strip passthrough args after " -- ", then
    # collect --flag tokens
    flags=$(sed 's/ -- .*$//' "$file" | grep -oE '\-\-[a-z][a-z-]+' | sort -u)

    for flag in $flags; do
        if ! grep -qE "(^|[^a-z-])$flag(\$|[^a-z-])" "$WORKDIR/help.all"; then
            echo "FAIL: $file mentions $flag which is not in the --help of any command it names" >&2
            failures=$((failures + 1))
        fi
    done
done

if [ "$failures" -gt 0 ]; then
    echo "check-flags.sh: $failures unknown flag(s) — skills have drifted from the CLI" >&2
    exit 1
fi

echo "check-flags.sh: all documented flags exist in live --help output"
