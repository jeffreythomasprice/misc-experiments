package main

import (
	"shared"
	"syscall/js"
	"time"

	"github.com/rs/zerolog/log"
)

func Reload() {
	go func() {
		_, receive := WebsocketClientStayConnectedForever[shared.WSMsgReloadClientToServer, shared.WSMsgReloadServerToClient](WebsocketUrl(shared.ReloadPath))
		var lastKey *string = nil
		startTime := time.Now()
		go func() {
			for msg := range receive {
				log.Info().Str("key", msg.Key).Msg("connected to server")
				if lastKey == nil {
					lastKey = &msg.Key
					if time.Since(startTime) > time.Second*3 {
						log.Info().Msg("first time connected to server, but too much time has passed, reloading just in case")
						reload()
					}
				} else if *lastKey != msg.Key {
					log.Info().Msg("server key changed, reloading")
					reload()
				}
			}
		}()
	}()
}

func reload() {
	js.Global().Get("location").Call("reload")
}
