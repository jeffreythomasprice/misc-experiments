.PHONY = \
	all \
	build \
	clean

OUT_DIR = bin
OUT_NAME = libexperiment.so

all: build

build:
	mkdir -p $(OUT_DIR)
	clang -shared -fPIC -o $(OUT_DIR)/$(OUT_NAME) src/lib.c
# 	mkdir -p ../c-lib-nif-manual/bin
# 	cp $(OUT_DIR)/$(OUT_NAME) ../c-lib-nif-manual/bin/
	mkdir -p ../gleam-manual/priv
	cp $(OUT_DIR)/$(OUT_NAME) ../gleam-manual/priv/

clean:
	rm -rf $(OUT_DIR)