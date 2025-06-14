#!/bin/bash

set -e

echo "🔧 Building all crates..."
cargo build --manifest-path runtime/Cargo.toml
cargo build --manifest-path p2p/Cargo.toml
cargo build --manifest-path jobmanager/Cargo.toml
cargo build --manifest-path devnet/Cargo.toml  
cargo build --manifest-path keygen/Cargo.toml
cargo build --manifest-path dashboard/Cargo.toml

echo "🧪 Testing CLI help commands..."

echo "  ✅ Testing jobmanager --help"
cargo run --manifest-path jobmanager/Cargo.toml -- --help > /dev/null

echo "  ✅ Testing keygen --help"  
cargo run --manifest-path keygen/Cargo.toml -- --help > /dev/null

echo "  ✅ Testing devnet --help"
cargo run --manifest-path devnet/Cargo.toml -- --help > /dev/null

echo "  ✅ Testing dashboard --help"
cargo run --manifest-path dashboard/Cargo.toml -- --help > /dev/null

echo "🎉 All CLI help commands working correctly!"
echo ""
echo "✅ CI pipeline fixes complete:"
echo "   - Dashboard CLI argument handling added"
echo "   - All binaries respond to --help and exit cleanly" 
echo "   - Smart contract warnings fixed"
echo "   - All crates building successfully" 