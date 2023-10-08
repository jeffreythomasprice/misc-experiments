package main

import (
	"client/websockets"
	"log/slog"
	"shared"
	"syscall/js"
	"time"
)

func liveReload() {
	var lastMessage *string = nil
	websockets.NewBuilder().
		OnTextMessage(func(value string) {
			slog.Debug("live reload message", "msg", value)
			if lastMessage == nil {
				lastMessage = &value
			} else if *lastMessage != value {
				slog.Info("server restart detected, reloading")
				js.Global().Get("location").Call("reload")
			}
		}).
		ReconnectStrategy(websockets.ConstantDelay(time.Second * 2)).
		Build(shared.LiveReloadPath)
}
