.PHONY = \
	all \
	build \
	clean \
	run

all: build

build:
	templ generate
	go build -o bin/experiment ./...

clean:
	rm -rf bin server/generated
	find . -name '*_templ.go' -delete

run: build
	bin/experiment

watch:
	watchexec -r make run