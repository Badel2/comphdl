#!/bin/bash
cd $(dirname $0)/..

# Clear caches because cargo-web breaks sometimes
rm -rf .cache
rm -rf dist
mkdir -p dist
cp -rf static/comphdl_examples/ dist/
PAGES_URL="/comphdl/demo/nightly"
LOCAL_URL="/"
# deploy to PAGES_URL if on travis, deploy to / if building locally
PUBLIC_URL=$([ -z "$TRAVIS_REPO_SLUG" ] && echo "$LOCAL_URL" || echo "$PAGES_URL")
if [ -z "$TRAVIS_REPO_SLUG" ]; then
    :
else
    while [ 1 ]; do sleep 5m && pidof cargo > /dev/null && echo "travis_wait: cargo still running"; done &
fi
parcel build static/index.html --log-level 4 --public-url $PUBLIC_URL

if [ -f dist/index.html ]; then
    # Deploy to demo/nightly folder
    mkdir -p demo
    rm -rf demo/nightly
    cp -rf dist demo/nightly
    exit 0
else
    # Assume parcel failed
    exit 1
fi

