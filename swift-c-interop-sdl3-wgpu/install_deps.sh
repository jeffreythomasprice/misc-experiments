rm -rf \
	.deps \
	*.so \
	*.so.* \
	Sources/CSDL/SDL3 \
	Sources/CWGPU/webgpu

mkdir -p .deps
pushd .deps

wget https://github.com/libsdl-org/SDL/releases/download/release-3.2.22/SDL3-3.2.22.tar.gz
tar -xvf SDL3-3.2.22.tar.gz
mkdir SDL3-build
pushd SDL3-build
cmake ../SDL3-3.2.22
make -j12
cp libSDL3.so* ../../
popd
cp -r SDL3-3.2.22/include/SDL3/ ../Sources/CSDL/

wget https://github.com/gfx-rs/wgpu-native/releases/download/v25.0.2.2/wgpu-linux-x86_64-release.zip
unzip wgpu-linux-x86_64-release.zip -d wgpu-linux-x86_64-release
cp wgpu-linux-x86_64-release/lib/libwgpu_native.so* ../
cp -r wgpu-linux-x86_64-release/include/webgpu/ ../Sources/CWGPU/

popd
