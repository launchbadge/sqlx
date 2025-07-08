#!/bin/bash

# This grabs a pre-compiled version of the extension used in this
# example, and stores it in a temporary directory. That's a bit
# unusual. Normally, any extensions you need will be installed into a
# directory on the library search path, either by using the system
# package manager or by compiling and installing it yourself.

mkdir /tmp/sqlite3-lib && wget -O /tmp/sqlite3-lib/ipaddr.so https://github.com/nalgeon/sqlean/releases/download/0.15.2/ipaddr.so
