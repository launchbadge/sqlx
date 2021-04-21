#!/usr/bin/env bash

# This script scans the project for `mod.rs` files and exits with a nonzero code if it finds any.
#
# You can also call it with `--fix` to replace any `mod.rs` files with their 2018 edition equivalents.
# The new files will be staged for commit for convenience.

FILES=$(find ./ -name mod.rs -print)

if [[ -z $FILES ]]; then
  exit 0
fi

if [ "$1" != "--fix" ]; then
  echo 'This project uses the Rust 2018 module style. mod.rs files are forbidden.'
  echo "Execute \`$0 --fix\` to replace these with their 2018 equivalents and stage for commit."
  echo 'Found mod.rs files:'
  echo "$FILES"
  exit 1
fi

echo 'Fixing Rust 2018 Module Style'

while read -r file; do
  dest="$(dirname $file).rs"
  echo "$file -> $dest"
  mv $file $dest
  git add $dest
done <<< $FILES

