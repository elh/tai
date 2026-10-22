#!/usr/bin/env bash
# Compile the generated module, then call it from a separate Swift 6 executable.
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/swift.sh
output="$PWD/target/swift"
swift_flags=(-swift-version 6 -module-cache-path "$output/ModuleCache" \
    -I "$output" -Xcc "-fmodule-map-file=$output/TaiFFI.modulemap")
swiftc "${swift_flags[@]}" -parse-as-library -emit-library -emit-module -module-name Tai \
    "$output/Tai.swift" "$output/libtai.a" \
    -emit-module-path "$output/Tai.swiftmodule" -o "$output/libTai.dylib"
swiftc "${swift_flags[@]}" -L "$output" -lTai \
    -Xlinker -rpath -Xlinker "$output" tests/swift/main.swift -o "$output/smoke"
"$output/smoke"
