#!/bin/bash

LIBS=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.targets[] | .kind[] == "lib") | .name' | tr '\n' ' ')
for lib in $LIBS; do
  cargo +nightly docs-rs -p "$lib"
done
