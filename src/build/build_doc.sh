#!/bin/bash
set -eu pipefail

# NOTE: this script is meant to be run from project root
# chmod +x src/build/build_doc.sh && src/build/build_doc.sh
cargo clean --doc
rm -rf docs/
cargo doc --no-deps
cp -r target/doc/ docs/
cp src/build/redirect.html docs/index.html
