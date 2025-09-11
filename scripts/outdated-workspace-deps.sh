#!/bin/bash

set -euo pipefail

# Script to check for outdated workspace dependencies in Cargo.toml
# Works around cargo-outdated issue: https://github.com/kbknapp/cargo-outdated/issues/360

CARGO_TOML="${1:-Cargo.toml}"

if [[ ! -f "$CARGO_TOML" ]]; then
    echo "Error: $CARGO_TOML not found"
    exit 1
fi

echo "Checking outdated workspace dependencies..."
echo

# Extract dependency names and versions from [workspace.dependencies] section
get_workspace_deps() {
    # Extract the [workspace.dependencies] section and process it
    sed -n '/^\[workspace\.dependencies\]/,/^\[/p' "$CARGO_TOML" | \
    grep -E '^[a-zA-Z0-9_-]+\s*=' | \
    while IFS= read -r line; do
        # Handle simple version format: name = "version"
        if [[ $line =~ ^([a-zA-Z0-9_-]+)[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
            echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
        # Handle complex format: name = { version = "version", ... }
        elif [[ $line =~ ^([a-zA-Z0-9_-]+)[[:space:]]*=.*version[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
            echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
        fi
    done
}

found_outdated=false

while read -r crate current_version; do
    [[ -z "$crate" ]] && continue
    
    # Get latest version from crates.io via cargo info
    # Extract from format: "version: 2.5.0 (latest 4.6.0)" -> "4.6.0"
    version_line=$(cargo info "$crate" --color never 2>/dev/null | grep "^version:" || echo "")
    
    # Try to extract from "(latest X.X.X)" first
    latest_version=$(echo "$version_line" | sed -n 's/.*(\s*latest \([^)]*\)).*/\1/p')
    
    # If no latest version found, fall back to the main version
    if [[ -z "$latest_version" ]]; then
        latest_version=$(echo "$version_line" | cut -d' ' -f2)
    fi
    
    if [[ -z "$latest_version" ]]; then
        echo "Warning: Could not fetch version for $crate"
        continue
    fi
    
    # Skip release candidates and deprecated versions
    if [[ "$latest_version" =~ -rc\. ]] || [[ "$latest_version" =~ \+deprecated$ ]]; then
        continue
    fi
    
    # Compare versions (simple string comparison should work for most semantic versions)
    if [[ "$current_version" != "$latest_version" ]]; then
        echo "$crate: $current_version → $latest_version"
        found_outdated=true
    fi
done < <(get_workspace_deps)

if [[ "$found_outdated" == false ]]; then
    echo "All workspace dependencies are up to date!"
fi