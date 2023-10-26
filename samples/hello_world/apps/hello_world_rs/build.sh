#!/bin/bash
set -e -o pipefail
cd $(dirname $0)

rustc test.rs --target wasm32-wasi \
  "-C" "link-arg=--initial-memory=65536" \
  "-C" "link-arg=-zstack-size=8192" \
  "-C" "link-arg=--export=__heap_base" \
  "-C" "link-arg=--export=__data_end" \
  "-C" "link-arg=--strip-all"
