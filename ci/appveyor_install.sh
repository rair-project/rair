#!/bin/sh
curl http://radare.mikelloc.com/get/1.6.0/radare2-w32-1.6.0.zip > radare2.zip
unzip radare2.zip
mv radare2-w32-1.7.0-git radare2
export LIBRARY_PATH=$(pwd)/radare2
cargo build --release --verbose
cp radare2/*.dll target/release/
