.PHONY = \
	all \
	build \
	clean \
	run

all: build

build:
	templ generate
	go build -o bin/experiment

clean:
	rm -rf bin
	rm -rf *_templ.go

run: build
	cd bin && ./experiment

watch:
	watchexec \
		--restart \
		make run