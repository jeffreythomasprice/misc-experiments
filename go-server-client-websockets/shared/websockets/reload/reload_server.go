//go:build !(js && wasm)

package reload

import (
	"log/slog"
	"net/http"
	"shared"
	"shared/websockets"
)

func NewAutoReloadServerHandlerFunc() http.HandlerFunc {
	server, result := websockets.NewWebsocketServerHandlerFunc()

	message := NewServerHelloMessage()
	slog.Debug(
		"sever auto reload websocket handler started",
		"time", message.ServerStartupTime().Format(shared.ISO8601_MILLIS_FORMAT),
	)

	go func() {
		for connection := range server.Incoming() {
			jsonConnection := websockets.NewJsonWebsocketConnection[ServerHelloMessage](connection)

			go func() {
				for range jsonConnection.Incoming() {
					// noop, clients can't talk about on this channel
				}
			}()

			jsonConnection.Send(message)
		}
	}()

	return result
}
