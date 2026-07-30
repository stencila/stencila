#!/bin/sh
# PostToolUse hook (Write|Edit): lint Stencila documents after edits.
#
# Reads the hook event JSON from stdin, and if the edited file has a
# Stencila extension, runs `stencila lint --as json` and returns the
# diagnostics as additionalContext so Claude can self-correct.
#
# Never exits 2: lint findings must not abort a turn.

set -u

# Honour the auto_lint user setting (default: on)
case "${CLAUDE_PLUGIN_OPTION_AUTO_LINT:-true}" in
    false | 0 | no) exit 0 ;;
esac

INPUT=$(cat)

# Extract tool_input.file_path from the event JSON: use jq when available,
# else a sed approximation (paths with embedded quotes are not supported)
if command -v jq > /dev/null 2>&1; then
    FILE=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty' 2> /dev/null)
else
    FILE=$(printf '%s' "$INPUT" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
fi

[ -n "${FILE:-}" ] || exit 0
[ -f "$FILE" ] || exit 0

# Only lint Stencila-relevant extensions
case "$FILE" in
    *.smd | *.myst | *.qmd) ;;
    *) exit 0 ;;
esac

# Resolve the CLI (same ladder as session-context.sh); silently no-op if absent
resolve_cli() {
    if [ -n "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH:-}" ] \
        && [ "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}" != "stencila" ] \
        && [ -x "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}" ]; then
        printf '%s' "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}"
        return 0
    fi
    if [ -x ./target/debug/stencila ] && [ -d rust/cli ]; then
        printf '%s' "$PWD/target/debug/stencila"
        return 0
    fi
    if command -v stencila > /dev/null 2>&1; then
        command -v stencila
        return 0
    fi
    for p in /usr/local/bin/stencila "$HOME/.local/bin/stencila" "$HOME/.cargo/bin/stencila"; do
        if [ -x "$p" ]; then
            printf '%s' "$p"
            return 0
        fi
    done
    return 1
}

CLI=$(resolve_cli) || exit 0

FIX=""
case "${CLAUDE_PLUGIN_OPTION_AUTO_FIX:-false}" in
    true | 1 | yes) FIX="--fix" ;;
esac

# shellcheck disable=SC2086
DIAGNOSTICS=$(NO_COLOR=1 "$CLI" lint "$FILE" $FIX --as json --yes 2> /dev/null)

# No output, or an empty diagnostics list: nothing to report
case "$DIAGNOSTICS" in
    "" | "[]" | "{}" | "null") exit 0 ;;
esac

# Bound what is fed back into context: a pathological document can produce
# very large diagnostics. Truncate on line boundaries after ~8000 bytes.
DIAGNOSTICS=$(printf '%s\n' "$DIAGNOSTICS" | awk -v file="$FILE" '
    { total += length($0) + 1 }
    total > 8000 {
        printf "... (diagnostics truncated; run stencila lint %s --as json for the full list)\n", file
        exit
    }
    { print }
')

json_escape() {
    printf '%s' "$1" | awk 'BEGIN{ORS="\\n"} {gsub(/\\/,"\\\\"); gsub(/"/,"\\\""); print}' | sed 's/\\n$//'
}

escaped=$(json_escape "Stencila lint diagnostics for $FILE (fix what is relevant; do not abort the current task):
$DIAGNOSTICS")
printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"%s"}}\n' "$escaped"
exit 0
