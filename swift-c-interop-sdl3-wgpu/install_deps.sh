set -e

# TODO only delete stuff first if a clean flag is provided
rm -rf .build .deps

mkdir -p .deps
pushd .deps

# TODO cache downloads

wget https://github.com/libsdl-org/SDL/releases/download/release-3.2.22/SDL3-3.2.22.tar.gz
tar -xvf SDL3-3.2.22.tar.gz
mkdir SDL3-build
pushd SDL3-build
cmake ../SDL3-3.2.22 -DCMAKE_BUILD_TYPE=Release -DSDL_STATIC=true -DSDL_SHARED=false
make -j12
popd

wget https://github.com/gfx-rs/wgpu-native/releases/download/v25.0.2.2/wgpu-linux-x86_64-release.zip
unzip wgpu-linux-x86_64-release.zip -d wgpu-linux-x86_64-release
rm wgpu-linux-x86_64-release/lib/libwgpu_native.so

popd
