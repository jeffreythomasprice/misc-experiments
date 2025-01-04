.PHONY = \
	all \
	build \
	clean \
	run

all: build

build:
	cd client && GOOS=js GOARCH=wasm go build -o ../server/generated/client.wasm
	cp $(shell go env GOROOT)/misc/wasm/wasm_exec.js server/generated/
	cd server && go build -o ../bin/experiment

clean:
	rm -rf server/generated
	rm -rf bin

run: build
	bin/experiment

watch:
	watchexec -r make run