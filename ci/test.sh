#!/bin/sh

(cd rio && cargo test --verbose)
(cd rtree && cargo test --verbose)

