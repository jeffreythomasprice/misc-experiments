.PHONY = \
	build \
	clean \
	run \
	watch

build:
	swift build

clean:
	rm -rf .build

run: build
	killall -9 carton || true
	carton dev \
		--skip-auto-open \
		--host 127.0.0.1 \
		--port 8000

watch:
	watchexec -r make run