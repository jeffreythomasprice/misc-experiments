#!/bin/bash

PID_FILE="./bin/.pid"

function build {
	make build
}

function run {
	./bin/server &
	local pid=$!
	echo "$pid" > "$PID_FILE"
	echo "new pid=$pid"
}

function start {
	echo "starting"
	
	if [ -f "$PID_FILE" ]; then
		echo "can't start, pid file exists at $PID_FILE"
		echo "pid file contents: $(cat "$PID_FILE")"
		return
	fi
	
	build
	run
	
	echo "started"
}

function stop {
	echo "stopping"

	if [ ! -f "$PID_FILE" ]; then
		echo "can't stop, no pid file at $PID_FILE"
		return
	fi
	local pid=$(cat "$PID_FILE")
	echo "stopping $pid"
	kill -TERM "$pid"
	rm -f "$PID_FILE"

	echo "stopped"
}

function restart {
	stop
	start
}

trap stop EXIT

start

inotifywait \
	-m \
	-r \
	-e create \
	-e move \
	-e delete \
	-e close_write \
	server \
	client \
	shared \
	web \
	|
	while read path action file; do
		echo "path=$path action=$action file=$file"
		restart
	done