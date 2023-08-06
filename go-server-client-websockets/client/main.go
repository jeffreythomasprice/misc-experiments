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

	// TODO JEFF demo
	host := js.Global().Get("window").Get("location").Get("host")
	wsUrl := fmt.Sprintf("ws://%v/ws", host)
	connection, err := websockets.NewWebsocketConnection(wsUrl)
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

	reload.StartAutoReloadClient(fmt.Sprintf("ws://%v/ws/autoreload", host))

	select {}
}
