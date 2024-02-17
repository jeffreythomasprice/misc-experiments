package main

import (
	"errors"
	"io"
	"net"

	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog/log"
	"golang.org/x/net/websocket"
)

func WebsocketHandler[Send, Receive any](f func(send chan<- Send, receive <-chan Receive)) echo.HandlerFunc {
	return func(c echo.Context) error {
		log := log.With().Str("remoteAddr", c.Request().RemoteAddr).Logger()

		send := make(chan Send)
		receive := make(chan Receive)

		websocket.Handler(func(ws *websocket.Conn) {
			defer ws.Close()
			defer log.Info().Msg("ws disconnect")
			log.Info().Msg("ws connected")

			go func() {
				defer close(send)
				defer close(receive)
				for {
					var msg Receive
					if err := websocket.JSON.Receive(ws, &msg); err != nil {
						if errors.Is(err, net.ErrClosed) || errors.Is(err, io.EOF) {
							return
						}
						log.Error().Err(err).Msg("failed to deserialize incoming message")
						continue
					}
					receive <- msg
				}
			}()

			go func() {
				for {
					for msg := range send {
						if err := websocket.JSON.Send(ws, &msg); err != nil {
							log.Error().Err(err).Msg("failed to serialize outgoing message")
						}
					}
				}
			}()

			f(send, receive)
		}).
			ServeHTTP(c.Response(), c.Request())
		return nil
	}
}
