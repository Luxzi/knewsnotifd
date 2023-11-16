#!/bin/bash
cargo build --release

# Build then run
# docker run --rm -it $(docker build -q .)

docker build