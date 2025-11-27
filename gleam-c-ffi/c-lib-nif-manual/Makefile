.PHONY = \
	all \
	build \
	clean

ERL_ROOT = "/home/linuxbrew/.linuxbrew/Cellar/erlang/28.2/lib/erlang/erts-16.1.2"

OUT_DIR = bin
OUT_NAME = libexperimentnif.so

all: build

build:
	mkdir -p $(OUT_DIR)
	clang -shared -fPIC -o $(OUT_DIR)/$(OUT_NAME) -I$(ERL_ROOT)/include -I../c-lib/src -L../c-lib/bin -lexperiment src/nif.c
	mkdir -p ../gleam-manual/priv
	cp $(OUT_DIR)/$(OUT_NAME) ../gleam-manual/priv/

clean:
	rm -rf $(OUT_DIR)