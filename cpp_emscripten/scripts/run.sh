#!/bin/bash

SRC_ROOT="$1"

echo "run begin"

function cleanup {
	echo "cleanup"
	echo "killing http server $serverPid"
	kill $serverPid
}
trap cleanup EXIT

emrun index.html --port 8000 --no_browser &
# python3 -m http.server 8000 &
serverPid=$!
echo "http server pid $serverPid"

while inotifywait -r -e modify,create,delete,move "$SRC_ROOT/src" "$SRC_ROOT/static"; do
	echo "rebuilding"
	make client
done

echo "run end"
