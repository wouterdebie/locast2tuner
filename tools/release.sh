#!/bin/bash
set -e
cargo bump patch
cargo check
new_version=$(grep ^version Cargo.toml | awk '{print $3}' | sed s/\"//g)
git commit -am "New version ${new_version}"
echo "Releasing version v${new_version}"
git tag v${new_version}
git push && git push --tags
