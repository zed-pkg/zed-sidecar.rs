#!/bin/sh
set -eu

target="${ZED_PKG_TEST_TARGET:?ZED_PKG_TEST_TARGET is required}"

cargo test --manifest-path "$target/Cargo.toml" --all-targets --locked
test -x "$target/target/release/zed-sidecar"
