#!/usr/bin/env sh
set -ex

VERSION=$1

if [ -z "$VERSION" ]
then
  echo "USAGE: ./prep-release.sh <version>"
  exit 1
fi

cargo set-version -p sqlx-rt "$VERSION"
cargo set-version -p sqlx-core "$VERSION"
cargo set-version -p sqlx-macros "$VERSION"
cargo set-version -p sqlx "$VERSION"
cargo set-version -p sqlx-cli "$VERSION"