#!/usr/bin/env bash
# Generate typed Swift bindings and a static library; optionally package iOS slices.
set -euo pipefail
cd "$(dirname "$0")/.."
root="$PWD"
output="$root/target/swift"
mode="${1:-macos}"
case "$mode" in macos|ios) ;; *) echo "usage: $0 [macos|ios]" >&2; exit 2 ;; esac

cargo build --locked --release --lib --features uniffi --target-dir "$root/target"
cargo run --locked --features bindgen --bin uniffi-bindgen --target-dir "$root/target" -- \
    generate "$root/target/release/libtai.dylib" --language swift \
    --config "$root/uniffi.toml" --out-dir "$output"
cp "$root/target/release/libtai.a" "$output/libtai.a"

if [[ "$mode" == ios ]]; then
    rustup target add aarch64-apple-ios aarch64-apple-ios-sim
    for target in aarch64-apple-ios aarch64-apple-ios-sim; do
        cargo build --locked --release --lib --features uniffi --target "$target" --target-dir "$root/target"
    done
    mkdir -p "$output/Headers"
    cp "$output/TaiFFI.h" "$output/Headers/"
    cp "$output/TaiFFI.modulemap" "$output/Headers/module.modulemap"
    # Only this generated product is replaced on repeat builds.
    rm -rf "$output/TaiFFI.xcframework"
    xcodebuild -create-xcframework \
        -library "$root/target/aarch64-apple-ios/release/libtai.a" -headers "$output/Headers" \
        -library "$root/target/aarch64-apple-ios-sim/release/libtai.a" -headers "$output/Headers" \
        -output "$output/TaiFFI.xcframework"
fi
