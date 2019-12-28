#!/usr/bin/env bash

# Get current directory (of this script)
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Run schema file
psql -d "$DATABASE_URL" -f schema.sql
