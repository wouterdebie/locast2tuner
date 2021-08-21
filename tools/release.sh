#!/bin/bash
set -e
level=${1:-patch}
echo "Releasing.."
echo "=== cargo check ==="
cargo check
echo "=== cargo clippy ==="
cargo clippy -- -D warnings
echo "=== cargo test ==="
cargo test
echo "=== cargo bump ${level} ==="
cargo bump ${level}
echo "=== cargo check ==="
cargo check
echo "=== git commit ==="
new_version=$(cat Cargo.toml | grep -E '^version' | cut -d'"' -f2)
git commit -am "Release ${new_version}"
echo "=== Tagging version v${new_version} ==="
git tag v${new_version}
echo "=== git push ==="
git push && git push --tags
