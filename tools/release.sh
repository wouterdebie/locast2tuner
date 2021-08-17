#!/bin/bash
set -e
echo "Releasing.."
echo "=== cargo check ==="
cargo check
echo "=== cargo clippy ==="
cargo clippy -- -D warnings
echo "=== cargo test ==="
cargo test
echo "=== cargo bump ==="
cargo bump patch
echo "=== cargo check ==="
cargo check
echo "=== git commit ==="
new_version=$(cat Cargo.toml | grep -E '^version' | cut -d'"' -f2)
git commit -am "Release ${new_version}"
echo "=== Tagging version v${new_version} ==="
git tag v${new_version}
echo "=== git push ==="
git push && git push --tags
