#!/bin/bash
set -e

if [ $COV ]; then
  set +e # cargo install might fail because package already exist in cache
  cargo install grcov
  set -e
  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
  cargo test --all $CARGO_OPTIONS # testing
  grcov --llvm target/debug -t lcov  > coverage.info
  bash <(curl -s https://codecov.io/bash) -f coverage.info
  cargo test --all
fi

if [ $DOC ]; then
  set +e # cargo install might fail because package already exist in cache
  cargo install cargo-deadlinks -q
  set -e
  cargo doc  --no-deps --all # building docs
  cargo rustdoc -- -Z unstable-options --enable-index-page
  cargo deadlinks --check-http --dir target/doc #testing docs
  if [ $TRAVIS_BRANCH = master ] && [ $TRAVIS_PULL_REQUEST = false ]; then
    pip install ghp-import --user
    ghp-import -n target/doc
    git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
  fi
fi
