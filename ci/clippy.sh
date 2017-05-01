#!/bin/sh
cargo install clippy
for d in r_*; do
	cd $d
	echo in $d
	cargo clippy
	cd ../
done
echo in rair
cargo clippy
