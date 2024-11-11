#!/bin/bash

set -e

echo "] cargo build"
cargo build

if [ ! -f ./kls ]; then
    echo "] ln -s ./target/debug/kls ./kls"
    ln -s ./target/debug/kls ./kls
fi
