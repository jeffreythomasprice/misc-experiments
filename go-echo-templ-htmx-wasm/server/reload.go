package main

import (
	"shared"
	"time"

	"github.com/labstack/echo/v4"
)

func ReloadHandler() echo.HandlerFunc {
	key := time.Now().Format(time.RFC3339Nano)
	return WebsocketHandler(func(send chan<- shared.WSMsgReloadServerToClient, receive <-chan shared.WSMsgReloadClientToServer) {
		send <- shared.WSMsgReloadServerToClient{
			Key: key,
		}

		for range receive {
		}
	})
}
