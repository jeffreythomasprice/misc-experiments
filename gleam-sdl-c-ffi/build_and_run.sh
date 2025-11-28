#!/bin/bash

set -e

pushd sdl || exit 1
make
popd || exit 1

export LD_LIBRARY_PATH="$(pwd)/sdl/deps/SDL3/build"
gleam run
