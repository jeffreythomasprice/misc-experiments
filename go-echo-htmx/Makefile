.PHONy: \
	all \
	build \
	clean \
	run \
	watch

all: build

build:
	go build -o bin/experiment ./...

clean:
	rm -rf bin

run:
	go run ./...

watch:
	watchexec \
		--ignore bin \
		-r \
		go run ./...
