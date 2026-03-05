#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"

# Generate changelog
git-cliff --tag "v$VERSION" -o CHANGELOG.md

# Update optionalDependencies versions in package.json
sed -i.bak -E 's/("langmail-(win32|darwin|linux)-[^"]+": )"[^"]+"/\1"'"$VERSION"'"/g' packages/langmail/package.json
rm -f packages/langmail/package.json.bak

git add CHANGELOG.md packages/langmail/package.json
