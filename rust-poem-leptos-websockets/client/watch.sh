#!/bin/bash

npx --yes concurrently --raw \
	"watchexec npx --yes tailwindcss -i ./index.css -o ./generated/index.css" \
	"trunk serve"
