#!/bin/sh
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:armv7-musleabihf'  && rust-musl-builder cargo build
docker build -t hainz98/smbackup -f Dockerfile-small .