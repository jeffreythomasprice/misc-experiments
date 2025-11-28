.PHONY = \
	all \
	build \
	clean

ERL_ROOT = "/home/linuxbrew/.linuxbrew/Cellar/erlang/28.2/lib/erlang/erts-16.1.2"

OUT_DIR = bin
OUT_NAME = libsdl.so

all: build

build:
	mkdir -p $(OUT_DIR)
	clang \
		-shared \
		-fPIC \
		-o $(OUT_DIR)/$(OUT_NAME) \
		-I$(ERL_ROOT)/include \
		-Ideps/SDL3/SDL3-3.2.26/include \
		-Ldeps/SDL3/build \
		-lSDL3 \
		src/lib.c

clean:
	rm -rf $(OUT_DIR)