.PHONY: \
	all \
	run \
	build \
	clean \
	assets \
	client \
	server

all: build

run: build
	./bin/server

build: assets client server

clean:
	rm -rf bin
	rm -rf server/embed

assets:
	mkdir -p server/embed
	rsync -a --delete assets/ server/embed/ --exclude client.wasm --exclude wasm_exec.js
	rsync $(shell go env GOROOT)/misc/wasm/wasm_exec.js server/embed/

client:
	GOOS=js GOARCH=wasm go build -o server/embed/client.wasm ./client

server: assets client
	go build -o bin/server ./server
