#!/bin/bash

set -e

echo "] ./build.sh"
./build.sh
echo "] ./kls $@"
./kls $@

