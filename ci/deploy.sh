#! /bin/sh

src=$(pwd)
cargo build --release
cd target/release/
tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
cd $src
