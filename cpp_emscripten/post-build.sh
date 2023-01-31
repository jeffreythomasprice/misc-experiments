#!/bin/bash

echo "post-build begin"

CMAKE_SOURCE_DIR="$1"
CMAKE_BINARY_DIR="$2"

echo "copying static files"
rsync -a "$CMAKE_SOURCE_DIR/static/" "$CMAKE_BINARY_DIR/"

echo "post-build end"
