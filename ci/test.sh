#!/bin/sh
set -e

if [ $COV ]; then
  set +e # cargo install might fail because package already exist in cache
  cargo install cargo-tarpaulin
  set -e
  cargo test --all --doc # testing doc tests
  cargo tarpaulin --ignore-tests -l --all --out Xml # testing and code coveage
  curl -s https://codecov.io/bash | bash
else
  cargo test --all
fi

if [ $DOC ]; then
  set +e # cargo install might fail because package already exist in cache
  cargo install cargo-deadlinks -q
  set -e
  cargo doc  --no-deps --all # building docs
  cargo rustdoc -- -Z unstable-options --enable-index-page
  cargo deadlinks --check-http --dir target/doc #testing docs
  if [ $TRAVIS_BRANCH = master ]; then
    pip install ghp-import --user
    ghp-import -n target/doc
    git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
  fi
fi
