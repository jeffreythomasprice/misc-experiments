package main

import (
	"fmt"
	"log/slog"
	"shared"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/olahol/melody"
)

func liveReload(e *echo.Echo) {
	m := melody.New()

	msg := fmt.Sprintf("%v", time.Now().Format(time.RFC3339))

	m.HandleConnect(func(s *melody.Session) {
		slog.Debug("client live reload websocket connected", "remote addr", s.RemoteAddr())

		s.Write([]byte(msg))
	})

	e.GET(shared.LiveReloadPath, func(c echo.Context) error {
		return m.HandleRequest(c.Response(), c.Request())
	})
}
