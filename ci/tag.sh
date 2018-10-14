#!/bin/bash
cd $(dirname $0)/..

# Safely copy a nightly deploy to a vXX folder
TAG_VERSION="v10"
NIGHTLY_URL="/comphdl/demo/nightly"
TAG_URL="/comphdl/demo/$TAG_VERSION"
cp -rf demo/nightly demo/$TAG_VERSION
find demo/$TAG_VERSION -type f -print0 | xargs -0 sed -i "s,$NIGHTLY_URL,$TAG_URL,g"

