#!/bin/sh
curl http://bin.rada.re/radare2-w32-1.4.0.zip > radare2-w32-1.4.0.zip
unzip radare2-w32-1.4.0.zip
export LIBRARY_PATH=$(pwd)/radare2-w32-1.4.0
cargo build --release --verbose
cp radare2-w32-1.4.0/*.dll target/release/
