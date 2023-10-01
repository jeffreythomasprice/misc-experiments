package main

import (
	"client/websockets"
	"context"
	"syscall/js"
	"time"
)

func liveReload(url string) {
	_, incoming := websockets.NewBuilder(url).
		Reconnect(websockets.Every(time.Second)).
		Build(context.Background())
	var lastMsg *string = nil
	for msg := range incoming {
		if msg.IsTextMessage() {
			if lastMsg == nil {
				s := msg.Text()
				lastMsg = &s
			} else if msg.Text() != *lastMsg {
				js.Global().Get("location").Call("reload")
			}
		}
	}
}
