#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cargo build --release

sysroot="$(rustc --print sysroot)"
objcopy="$sysroot/lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy"

"$objcopy" \
  -O binary \
  target/aarch64-unknown-none/release/xiaojin-os \
  target/aarch64-unknown-none/release/xiaojin-os.bin

ls -lh \
  target/aarch64-unknown-none/release/xiaojin-os \
  target/aarch64-unknown-none/release/xiaojin-os.bin
