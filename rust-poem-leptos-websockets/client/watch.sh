#!/bin/bash

npx --yes concurrently \
	"watchexec npx --yes tailwindcss -i ./index.css -o ./generated/index.css" \
	"trunk serve"
