
.PHONY : \
	all \
	build \
	exe \
	lib \
	clean

all: build

run: lib
	@cd executable && gleam run

build: lib exe

exe: lib
	@cd executable && gleam build

lib:
	@cd lib && cargo build
	@mkdir -p executable/priv
	@cp ./lib/target/debug/librust_lib.so executable/priv

clean:
	@rm -rf \
		lib/target \
		executable/priv \
		executable/build