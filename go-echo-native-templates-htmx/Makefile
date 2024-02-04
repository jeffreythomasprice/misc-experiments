.PHONY = \
	all \
	build \
	clean \
	run

all: build

build:
	go build -o bin/experiment

clean:
	rm -rf bin

run: build
	cd bin && ./experiment

watch:
	watchexec \
		--restart \
		make run