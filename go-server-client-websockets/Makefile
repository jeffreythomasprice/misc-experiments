.PHONY: \
	all \
	build \
	clean \
	client \
	server

all: build

build: client server

clean:
	rm -rf bin

client:
	cd client && GOOS=js GOARCH=wasm go build -o ../bin/web/client.wasm
	cp $(shell go env GOROOT)/misc/wasm/wasm_exec.js bin/web/
	rsync -a web/ bin/web/

server:
	cd server && go build -o ../bin/server
