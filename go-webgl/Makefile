.PHONY = \
	all \
	build \
	clean \
	run

all: build

build:
	mkdir -p server/generated
	cp $(shell go env GOROOT)/misc/wasm/wasm_exec.js server/generated/
	cd client && GOOS=js GOARCH=wasm go build -o ../server/generated/client.wasm
	cd server && go build -o ../bin/server

clean:
	rm -rf bin server/generated

run: build
	bin/server

watch:
	watchexec -r make run