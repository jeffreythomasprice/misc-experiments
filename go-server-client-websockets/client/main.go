package main

import (
	"fmt"
	"log/slog"
	"os"
	"shared"
	"shared/websockets"
	"shared/websockets/reload"
	"syscall/js"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level:     slog.LevelDebug,
			AddSource: false,
		},
	)))

	host := js.Global().Get("window").Get("location").Get("host")
	reload.StartAutoReloadClient(fmt.Sprintf("ws://%v/ws/autoreload", host))

	// TODO JEFF demo
	connection, err := websockets.NewWebsocketConnection(fmt.Sprintf("ws://%v/ws", host))
	if err != nil {
		panic(err)
	}
	jsonConnection := websockets.NewJsonWebsocketConnection[shared.Message](connection)
	go func() {
		for message := range jsonConnection.Incoming() {
			slog.Info("incoming message", "text", message.Message)
		}
	}()
	jsonConnection.Send(shared.Message{
		Message: "Hello from client",
	})

	body, err := GetDomElementByQuerySelector("body")
	if err != nil {
		panic(err)
	}
	if err := body.RemoveAllChildren(); err != nil {
		panic(err)
	}
	content, err := NewDomElementFromHtmlString(`
		<p>foo</p>
		<p>bar</p>
		<p>baz</p>
	`)
	if err != nil {
		panic(err)
	}
	for _, c := range content {
		if err := body.AppendChild(c); err != nil {
			panic(err)
		}
	}

	select {}
}
