#!/bin/bash
set -e
cargo bump patch
cargo check
cargo build --release
cargo test
./tools/gh-md-toc --insert --no-backup --hide-footer README.
new_version=$(grep ^version Cargo.toml | awk '{print $3}' | sed s/\"//g)
git commit -am "New version ${new_version}"
echo "Releasing version v${new_version}"
git tag v${new_version}
git push && git push --tags
