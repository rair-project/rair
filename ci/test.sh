#!/bin/sh

cd test
for f in *.sh; do sh "$f"; done
cd ../r_search
cargo test --verbose
