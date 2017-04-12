#!/bin/sh
git clone https://github.com/radare/radare2.git
cd radare2
sys/install.sh
cd ../
cargo build --release --verbose
