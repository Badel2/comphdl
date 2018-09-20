#!/bin/sh
# This script is meant to be run from the project dir, not
# from inside the "ci" dir, so to make sure we cd to the "ci"
# dir which contains this file, and then to ".."
cd $(dirname $0)/..

ci/create_all_branches.sh &&
git checkout gh-pages &&
git pull origin master &&
ci/install_cargo_web.sh &&
ci/build_demo.sh &&
git add demo/nightly &&
git commit -m 'Nighlty demo' &&
# We must somehow preserve the demo/v07 folder...
git push -q https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
