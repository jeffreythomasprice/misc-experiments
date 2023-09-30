#!/bin/bash

BIN='make run'
PID_FILE="./bin/.pid"

function getPid {
	if [[ ! -f "$PID_FILE" ]]; then
		return
	fi
	cat "$PID_FILE"
}

function setPid {
	mkdir -p "$(dirname "$PID_FILE")"
	echo -n "$1" > "$PID_FILE"
}

function clearPid {
	rm -f "$PID_FILE"
}

function start {
	local pid=$(getPid)
	if [[ ! -z "$pid" ]]; then
		echo "already running, can't start"
		exit 1
	fi
	$BIN &
	setPid $!
	echo "started pid $pid"
}

function stop {
	local pid=$(getPid)
	if [[ -z "$pid" ]]; then
		echo "already stopped, can't stop first"
		return
	fi
	kill "$pid"
	clearPid
	echo "stopped pid $pid"
}

function restart {
	stop
	start
}

start

trap stop EXIT

inotifywait \
	-mr ./ \
	--include '.*\.go' \
	-e create \
	-e delete \
	-e modify \
	|
	while read -r directory action file; do
		sleep 0.5
		restart
	done