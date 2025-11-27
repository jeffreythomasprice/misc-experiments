.PHONY = \
	all \
	build \
	clean \
	run

OUT_DIR = bin
OUT_NAME = experiment

all: build

build:
	@mkdir -p $(OUT_DIR)
	@nasm -f elf64 -o $(OUT_DIR)/$(OUT_NAME).o -l $(OUT_DIR)/$(OUT_NAME).lst main.s
	@ld -static -o $(OUT_DIR)/$(OUT_NAME) $(OUT_DIR)/$(OUT_NAME).o

clean:
	@rm -rf $(OUT_DIR)

run: build
	@./$(OUT_DIR)/$(OUT_NAME)
