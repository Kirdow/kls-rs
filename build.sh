#!/bin/bash

set -e

echo "] cargo build"
cargo build

echo "] cp ./target/debug/kls ./kls"
cp ./target/debug/kls ./kls

