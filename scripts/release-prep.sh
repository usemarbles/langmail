#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"

# Navigate to workspace root (script may be called from a crate subdirectory)
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Prevent duplicate runs (hook fires per crate)
MARKER="/tmp/.langmail-release-prep-$VERSION"
if [ -f "$MARKER" ]; then
    exit 0
fi
touch "$MARKER"

# Generate changelog
git-cliff --tag "v$VERSION" -o CHANGELOG.md

# Update version and optionalDependencies in package.json
PKG="packages/langmail/package.json"
sed -i.bak -E \
    -e 's/("version": )"[^"]+"/\1"'"$VERSION"'"/' \
    -e 's/("langmail-(win32|darwin|linux)-[^"]+": )"[^"]+"/\1"'"$VERSION"'"/g' \
    "$PKG"
rm -f "$PKG.bak"

git add CHANGELOG.md "$PKG"
