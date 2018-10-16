#!/bin/bash
# This script is meant to be run from the project dir, not
# from inside the "ci" dir, so to make sure we cd to the "ci"
# dir which contains this file, and then to ".."
cd $(dirname $0)/..

NODE_VERSION="10"

ci/create_all_branches.sh &&
git checkout gh-pages &&
git pull origin master &&
ci/install_cargo_web.sh &&
source ~/.nvm/nvm.sh &&
nvm install $NODE_VERSION &&
npm install -g parcel-bundler &&
# https://github.com/npm/npm/issues/13528#issuecomment-396522166
npm install netlistsvg/ &&
npm install &&
ci/parcel.sh &&
git add demo/nightly &&
git commit -qm 'Nighlty demo' &&
# We must somehow preserve the demo/v07 folder...
git push -q https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages &&
exit 0

# If any command fails, exit with error status
exit 1

