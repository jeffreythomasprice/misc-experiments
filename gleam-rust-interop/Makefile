
.PHONY : \
	all \
	build \
	exe \
	lib \
	clean

all: build

run: lib
	cd executable && gleam run

build: lib exe

exe: lib
	cd executable && gleam build

lib:
	cd lib && cargo build
	bun x -y mkdirp executable/priv
	bun x -y shx cp ./lib/target/debug/librust_lib.so executable/priv

clean:
	bun x -y shx rm -rf \
		lib/target \
		executable/priv \
		executable/build