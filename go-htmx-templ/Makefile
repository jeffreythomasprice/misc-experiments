.PHONY: \
	all \
	build \
	run \
	watch \
	clean

all: build

build:
	templ generate
	go build -o bin/experiment .

run: build
	bin/experiment

watch:
	watchexec \
		-i "**/bin/**" \
		-i "**/*_templ.go" \
		-r \
		make run

clean:
	rm -rf bin