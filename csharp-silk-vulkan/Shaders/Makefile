.PHONY: \
	all \
	build \
	clean

SHADERS = $(patsubst %.vert,%.vert.spv,$(wildcard *.vert)) \
          $(patsubst %.frag,%.frag.spv,$(wildcard *.frag))

all: build

build: $(SHADERS)

%.spv: %
	glslc $< -o $@

clean:
	rm -rf *.spv