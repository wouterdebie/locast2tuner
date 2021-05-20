#!/bin/bash
set -e
cargo check
cargo clippy -- -D warnings
cargo test
cargo bump patch
cargo check
./tools/gh-md-toc --insert --no-backup --hide-footer README.md
new_version=$(grep ^version Cargo.toml | awk '{print $3}' | sed s/\"//g)
git commit -am "New version ${new_version}"
echo "Releasing version v${new_version}"
git tag v${new_version}
git push && git push --tags
