#!/bin/sh
if [ $COV ]; then
  cargo install cargo-tarpaulin
  cargo test --all --doc # testing doc tests
  cargo tarpaulin --ignore-tests -l --all --out Xml # testing and code coveage
  curl -s https://codecov.io/bash | bash
else
  cargo test --all
fi

if [ $DOC ]; then
  cargo install cargo-deadlinks
  cargo doc  # building docs
  cargo deadlinks --check-http --dir target/doc #testing docs
  pip install ghp-import --user
  echo "<meta http-equiv=refresh content=0;url=`echo $TRAVIS_REPO_SLUG | cut -d '/' -f 2`/index.html>" > target/doc/index.html
  ghp-import -n target/doc
  git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
fi
