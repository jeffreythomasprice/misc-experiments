package main

import (
	"context"
	"log/slog"
	"syscall/js"
)

func liveReload(url string) {
	var lastMessage *string = nil

	read, _ := NewWebsocket(context.Background(), url, &WebsocketClientOptions{Reconnect: true})

	go func() {
		for msg := range read {
			switch typedMsg := msg.(type) {
			case WebsocketTextMessageEvent:
				if lastMessage == nil {
					s := typedMsg.Data()
					lastMessage = &s
				} else {
					slog.Debug("reloading")
					js.Global().Get("window").Get("location").Call("reload")
				}
			}
		}
	}()
}
