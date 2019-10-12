#!/bin/sh
if [ $DOCOV ]; then
  cargo install cargo-tarpaulin
  cargo test --all --doc
  cargo tarpaulin --ignore-tests -l --all --out Xml
  curl -s https://codecov.io/bash | bash
else
  cargo test --all
fi
