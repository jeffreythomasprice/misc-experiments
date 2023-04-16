#!/bin/bash
mkdir -p bin/c-lib
pushd bin/c-lib
cmake ../../c-lib
make
popd