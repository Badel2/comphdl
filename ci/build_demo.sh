#!/bin/sh
echo 'Outdated, use ci/parcel.sh'
exit 1
#git checkout gh-pages
#git pull origin master
cd netlistsvg
npm install
npm run build-badel
cd ..
cargo +nightly web deploy --features stdweb
mkdir -p demo
rm -rf demo/nightly
cp -rf target/deploy demo/nightly
#git add demo
#git commit -am 'comphdl wasm demo'
#git push
#git checkout master
