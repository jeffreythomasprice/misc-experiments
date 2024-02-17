.PHONY = \
	all \
	build \
	clean \
	run \
	test \
	test-watch

all: build

build:
	go build -o bin/experiment .

clean:
	rm -rf bin

run: build
	bin/experiment

watch:
	watchexec -r make run

test:
	go test .

test-watch:
	watchexec -r make test