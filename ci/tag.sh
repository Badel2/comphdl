#!/bin/bash
cd $(dirname $0)/..

# First manually tag the master branch:

# git checkout master
# git tag blog-10
# git push --tags

# Now switch to the gh-pages branch and copy the nightly folder to v10:

# git fetch origin
# git checkout origin/gh-pages

# Safely copy a nightly deploy to a vXX folder
TAG_VERSION="v10"
NIGHTLY_URL="/comphdl/demo/nightly"
TAG_URL="/comphdl/demo/$TAG_VERSION"
cp -rf demo/nightly demo/$TAG_VERSION
find demo/$TAG_VERSION -type f -print0 | xargs -0 sed -i "s,$NIGHTLY_URL,$TAG_URL,g"

# git add demo/v10
# git commit -m 'Demo v10'

# The 'nightly demo' commit must always be the last one, as
# it will be rewritten by travis. Use an interactive rebase
# to swap the last two commits, and force-push to gh-pages:

# git rebase -i HEAD~2
# git push origin HEAD:gh-pages -f

# And done, back to master

# git checkout master
