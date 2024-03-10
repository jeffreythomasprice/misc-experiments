.PHONY = \
	build-native \
	clean \
	run \
	watch

build-native:
	swift build

clean:
	rm -rf .build

run:
	killall -9 carton || true
	carton dev \
		--skip-auto-open \
		--host 127.0.0.1 \
		--port 8000

watch:
	watchexec -r make run