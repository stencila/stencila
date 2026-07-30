#!/bin/sh
# SessionStart hook: detect a Stencila workspace and inject context.
#
# Contract: a project with no Stencila content pays nothing — exit 0 with
# empty stdout, fast (<100ms). Never blocks, never installs anything.

set -u

# --- Is this a Stencila project at all? (cheap checks first) ---------------

has_stencila_files() {
    [ -f stencila.toml ] || [ -f stencila.local.toml ] && return 0
    # Look for Stencila-flavoured documents, shallowly and bounded: a hit at
    # the top level or one directory down is enough. Avoid deep scans of
    # huge trees on the no-op path.
    for f in ./*.smd ./*.myst ./*.qmd ./*/*.smd ./*/*.myst ./*/*.qmd; do
        [ -e "$f" ] && return 0
    done
    return 1
}

has_stencila_files || exit 0

# --- Resolve the CLI -------------------------------------------------------

resolve_cli() {
    # 1. Explicit user configuration
    if [ -n "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH:-}" ] \
        && [ "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}" != "stencila" ] \
        && [ -x "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}" ]; then
        printf '%s' "${CLAUDE_PLUGIN_OPTION_STENCILA_PATH}"
        return 0
    fi
    # 2. Local debug build when working in the Stencila repo itself
    if [ -x ./target/debug/stencila ] && [ -d rust/cli ]; then
        printf '%s' "$PWD/target/debug/stencila"
        return 0
    fi
    # 3. PATH
    if command -v stencila > /dev/null 2>&1; then
        command -v stencila
        return 0
    fi
    # 4. Common install locations
    for p in /usr/local/bin/stencila "$HOME/.local/bin/stencila" "$HOME/.cargo/bin/stencila"; do
        if [ -x "$p" ]; then
            printf '%s' "$p"
            return 0
        fi
    done
    return 1
}

json_escape() {
    # Escape backslashes, double quotes, and newlines for a JSON string
    printf '%s' "$1" | awk 'BEGIN{ORS="\\n"} {gsub(/\\/,"\\\\"); gsub(/"/,"\\\""); print}' | sed 's/\\n$//'
}

emit() {
    escaped=$(json_escape "$1")
    printf '{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"%s"}}\n' "$escaped"
}

if ! CLI=$(resolve_cli); then
    emit "This project contains Stencila documents, but no stencila CLI was found. Run /stencila:doctor for install instructions. Do not attempt to install it without being asked."
    exit 0
fi

# --- Gather context (bounded; this path may take a little longer) ----------

VERSION=$(NO_COLOR=1 "$CLI" --version 2> /dev/null | head -1)

# Document inventory: bounded count and listing
DOCS=$(find . -path ./node_modules -prune -o -path './.*' -prune -o \
    \( -name '*.smd' -o -name '*.myst' -o -name '*.qmd' \) -print 2> /dev/null | head -20)
NDOCS=$(printf '%s\n' "$DOCS" | grep -c . || true)

# Kernel names only (table output is verbose); tolerate failure
KERNELS=$(NO_COLOR=1 "$CLI" kernels list --yes 2> /dev/null \
    | awk -F'│' 'NF>2 {split($2, a, " "); if (a[1] != "" && a[1] != "Name") printf "%s ", a[1]}')

CONTEXT="Stencila workspace detected.
CLI: $CLI ($VERSION). Always invoke it by this absolute path, non-interactively: NO_COLOR=1 $CLI <cmd> --yes
Documents ($NDOCS shown, .smd/.myst/.qmd):
$DOCS
Available kernels: ${KERNELS:-unknown}
Workspace config: $([ -f stencila.toml ] && echo 'stencila.toml present' || echo 'no stencila.toml (see /stencila:doctor or stencila init)')
The stencila plugin's skills cover authoring, execution, conversion, publishing and workspaces."

emit "$CONTEXT"
exit 0
