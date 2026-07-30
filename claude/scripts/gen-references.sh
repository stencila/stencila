#!/bin/sh
# Generate skill reference files from site/docs/documents/*.smd
#
# Strips site frontmatter, converts `smd demo` fences to plain `smd` fences,
# and drops trailing site-navigation sections ("Related guides", "Schema
# types"). Output is optimised for model consumption, not human reading.

set -e

cd "$(dirname "$0")/.."

SRC=../site/docs/documents
OUT=skills/authoring/references

# Source files to convert, in "source:target" form
DOCS="
basics:basics.md
code:code.md
execution:execution.md
include-call:include-call.md
math:math.md
figures:figures.md
tables:tables.md
lists:lists.md
admonitions:admonitions.md
sections:sections.md
metadata:metadata.md
media:media.md
notes:notes.md
"

mkdir -p "$OUT"
rm -f "$OUT"/*.md

for pair in $DOCS; do
    src="$SRC/$(echo "$pair" | cut -d: -f1).smd"
    out="$OUT/$(echo "$pair" | cut -d: -f2)"

    if [ ! -f "$src" ]; then
        echo "gen-references.sh: missing source $src" >&2
        exit 1
    fi

    {
        echo "<!-- Generated from site/docs/documents by claude/scripts/gen-references.sh. Do not edit. -->"
        echo
        awk '
            # Strip YAML frontmatter at the top of the file
            NR == 1 && /^---$/ { infront = 1; next }
            infront { if (/^---$/) infront = 0; next }

            # Stop at trailing site-navigation sections
            /^# Related guides/ || /^# Schema types/ { exit }

            # Convert demo fences to plain smd fences
            /^````*smd demo$/ { sub(/ demo$/, ""); print; next }

            { print }
        ' "$src"
    } > "$out"
done

echo "gen-references.sh: generated $(find "$OUT" -name '*.md' | wc -l | tr -d ' ') reference files in $OUT"
