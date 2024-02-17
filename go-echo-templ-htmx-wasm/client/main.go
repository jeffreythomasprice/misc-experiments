package main

import (
	"shared"

	"github.com/rs/zerolog/log"
)

func main() {
	shared.InitLogger()

	log.Debug().Msg("test")

	send, receive := WebsocketClientStayConnectedForever[shared.WebsocketClientToServerMessage, shared.WebsocketServerToClientMessage](WebsocketUrl("/ws"))
	go func() {
		for msg := range receive {
			log.Info().Str("msg", msg.Message).Msg("received message")
		}
	}()
	send <- shared.WebsocketClientToServerMessage{
		Message: "Hello from the client",
	}

	// wait forever
	select {}
}
