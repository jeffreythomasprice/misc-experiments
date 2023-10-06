.PHONY: \
	all \
	build \
	run \
	watch \
	clean

all: build

build:
	go build -o bin/experiment .

run: build
	bin/experiment

watch:
	watchexec \
		-i "**/bin/**" \
		-r \
		make run

clean:
	rm -rf bin