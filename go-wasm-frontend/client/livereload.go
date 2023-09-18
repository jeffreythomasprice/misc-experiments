package main

import (
	"context"
	"log/slog"
	"syscall/js"
)

func liveReload() {
	var lastMessage *string = nil
	WebsocketWithReconnect(context.Background(), "ws://localhost:8000/_liveReload", nil, func(message string) {
		if lastMessage == nil {
			lastMessage = &message
		} else if *lastMessage != message {
			slog.Debug("reloading")
			js.Global().Get("window").Get("location").Call("reload")
		}
	})
}
