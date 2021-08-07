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
echo "=== Updating TOC ==="
./tools/gh-md-toc --insert --no-backup --hide-footer README.md
new_version=$(grep ^version Cargo.toml | awk '{print $3}' | sed s/\"//g)
echo "=== git commit ==="
git commit -am "Release ${new_version}"
echo "=== Tagging version v${new_version} ==="
git tag v${new_version}
echo "=== git push ==="
git push && git push --tags
