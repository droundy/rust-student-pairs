#!/bin/sh

set -ev

cargo build --release --target=x86_64-unknown-linux-musl

scp target/x86_64-unknown-linux-musl/release/rust-student-pairs wbo@bingley.physics.oregonstate.edu:/srv/wbo/bin/pairs-new

ssh wbo@bingley.physics.oregonstate.edu mv bin/pairs-new bin/pairs
