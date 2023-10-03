.PHONY: \
	all \
	build \
	run \
	clean

all: build

build:
	go build -o bin/experiment .

run: build
	bin/experiment

clean:
	rm -rf bin