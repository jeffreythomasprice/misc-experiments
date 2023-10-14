.PHONY: \
	all \
	build \
	run \
	watch

all: build

build:
	swift build

run:
	swift run App

watch:
	watchexec -w Public -w Resources -w Sources -r make run