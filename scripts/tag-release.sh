#!/bin/bash
set -euo pipefail

VERSION=$(node -p "require('./package.json').version")
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

if [ "$VERSION" != "$CARGO_VERSION" ]; then
  echo "ERROR: version mismatch"
  echo "  package.json: $VERSION"
  echo "  Cargo.toml:   $CARGO_VERSION"
  echo ""
  echo "Update both to the same version before tagging."
  exit 1
fi

TAG="v$VERSION"

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "ERROR: tag $TAG already exists"
  exit 1
fi

echo "Creating tag: $TAG"
git tag -a "$TAG" -m "Release $TAG"
git push origin "$TAG"
echo "Done. GitHub Actions will build and publish."
