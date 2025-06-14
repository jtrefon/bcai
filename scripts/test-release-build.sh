#!/bin/bash

# Test script to validate release build process locally
# Usage: ./scripts/test-release-build.sh

set -e

echo "🔧 Testing BCAI Release Build Process..."
echo

# Build all binaries in release mode
echo "📦 Building all binaries in release mode..."
cargo build --release --manifest-path devnet/Cargo.toml
cargo build --release --manifest-path jobmanager/Cargo.toml
cargo build --release --manifest-path keygen/Cargo.toml
cargo build --release --manifest-path dashboard/Cargo.toml

echo "✅ All binaries built successfully"
echo

# Test all binaries
echo "🧪 Testing binary help commands..."
./devnet/target/release/devnet --help > /dev/null && echo "✅ devnet --help works"
./jobmanager/target/release/jobmanager --help > /dev/null && echo "✅ jobmanager --help works"
./keygen/target/release/keygen --help > /dev/null && echo "✅ keygen --help works"
./dashboard/target/release/dashboard --help > /dev/null && echo "✅ dashboard --help works"

echo

# Test packaging
echo "📦 Testing packaging process..."
mkdir -p test-release-package
cp devnet/target/release/devnet test-release-package/
cp jobmanager/target/release/jobmanager test-release-package/
cp keygen/target/release/keygen test-release-package/
cp dashboard/target/release/dashboard test-release-package/

echo "📁 Package contents:"
ls -la test-release-package/

# Create archive
cd test-release-package && tar -czf ../test-release.tar.gz *
cd ..

echo "✅ Created test-release.tar.gz ($(du -h test-release.tar.gz | cut -f1))"

# Cleanup
rm -rf test-release-package test-release.tar.gz

echo
echo "🎉 All tests passed! Release build process is working correctly."
echo "💡 You can now safely create a release or run the release workflow." 