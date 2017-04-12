#!/bin/sh

cd test
for f in *.sh; do bash "$f"; done
cd ../r_search
cargo test --verbose
cd ../r_config
cargo test --verbose
