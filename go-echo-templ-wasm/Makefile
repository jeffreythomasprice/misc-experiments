.PHONY: \
	all \
	build \
	run \
	watch \
	clean \
	client \
	server

all: build

build: client server

run: build
	bin/server

watch:
	watchexec \
		-i "**/server/assets/generated/**" \
		-i "**/bin/**" \
		-r \
		make run

clean:
	rm -rf bin
	rm -rf server/assets/generated

client:
	cd client && templ generate
	GOARCH=wasm GOOS=js go build -o server/assets/generated/client.wasm ./client
	rsync $(shell go env GOROOT)/misc/wasm/wasm_exec.js server/assets/generated/

server:
	go build -o bin/server ./server
