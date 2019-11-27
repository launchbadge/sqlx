#!/usr/bin/env bash

# Get current directory (of this script)
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Run SQL files in schema/ directory
psql -d "$DATABASE_URL" -f $DIR/schema/*.sql
