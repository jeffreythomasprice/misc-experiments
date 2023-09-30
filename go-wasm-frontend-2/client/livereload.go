package main

import (
	"log/slog"
	"syscall/js"
)

func liveReload(url string) {
	var lastMessage *string = nil
	NewWebsocketWithReconnect(url, nil, func(message string) {
		if lastMessage == nil {
			lastMessage = &message
		} else if *lastMessage != message {
			slog.Debug("reloading")
			js.Global().Get("window").Get("location").Call("reload")
		}
	})
}
