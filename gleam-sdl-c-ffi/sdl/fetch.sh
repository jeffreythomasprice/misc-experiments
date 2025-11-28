#!/bin/bash

set -e

function fetch {
	local src="$1"
	local dst="$2"
	local unzip="$3"
	if [ -f "$dst" ]; then
		echo "$dst already exists"
		return
	fi
	echo "downloading $src to $dst"
	wget "$src" -O "$dst"
	if [[ "$dst" =~ ".tar.gz" ]]; then
		mkdir -p "$unzip"
		tar xvf "$dst" -C "$unzip"
	fi
}

mkdir -p deps
pushd deps || exit 1
fetch https://github.com/libsdl-org/SDL/releases/download/release-3.2.26/SDL3-3.2.26.tar.gz SDL3.tar.gz SDL3
popd || exit 1

mkdir -p deps/SDL3/build
pushd deps/SDL3/build || exit 1
# cmake -DBUILD_SHARED_LIBS=OFF ../SDL3-3.2.26
cmake ../SDL3-3.2.26
make -j
popd || exit 1