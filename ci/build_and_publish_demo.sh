#!/bin/bash
# This script is meant to be run from the project dir, not
# from inside the "ci" dir, so to make sure we cd to the "ci"
# dir which contains this file, and then to ".."
cd $(dirname $0)/..

NODE_VERSION="10"

ci/create_all_branches.sh &&
ci/install_cargo_web.sh &&
source ~/.nvm/nvm.sh &&
nvm install $NODE_VERSION &&
npm install -g parcel-bundler &&
npm install &&
ci/parcel.sh &&
mv demo/nightly demo/nightly_ &&
git checkout -- . &&
git checkout gh-pages &&
rm -rf demo/nightly &&
mv demo/nightly_ demo/nightly &&
git add demo/nightly &&
git commit --amend -qm 'Nighlty demo' &&
# Force push to gh-pages rewriting the last commit
git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages &&
exit 0

# If any command fails, exit with error status
exit 1

