.PHONY = \
	all \
	build \
	clean \
	run

all: build

build:
	GOOS=js GOARCH=wasm go build -o server/generated/client.wasm client
	rsync $(shell go env GOROOT)/misc/wasm/wasm_exec.js server/generated/
	cd server && templ generate
	go build -o bin/server server

clean:
	rm -rf bin server/generated
	find . -name '*_templ.go' -delete

run: build
	bin/server

watch:
	watchexec -r make run