#!/bin/bash

CONFIGURATION=${1:-Debug}
echo "CONFIGURATION = $CONFIGURATION"

function start {
	echo "starting"
	if [ ! -z $pid ]; then
		echo "starting, but pid already exists, can't start"
		return
	fi
	dotnet run -c $CONFIGURATION &
	pid=$!
	echo "pid = $pid"
}

function stop {
	echo "stopping $pid"
	if [ -z $pid ]; then
		echo "no pid to stop"
		return
	fi
	kill $pid
	# TODO check to make sure it's really gone and kill if necessary
	pid=
	echo "stopped"
}

function restart {
	if [ ! -z $pid ]; then
		stop
	fi
	start
}

function cleanup {
	stop
}

trap cleanup EXIT

start

while inotifywait -r -e modify,create,delete,move --exclude 'bin/|obj/' ./; do
	restart
done
