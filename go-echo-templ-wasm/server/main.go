package main

import (
	"embed"
	"io/fs"
	"net/http"
	"shared"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

//go:embed assets
var assets embed.FS

func main() {
	shared.InitLogging(true)

	e := echo.New()

	e.Debug = true
	e.HideBanner = true
	e.HidePort = true

	e.Use(middleware.RequestLoggerWithConfig(middleware.RequestLoggerConfig{
		LogStatus: true,
		LogURI:    true,
		LogValuesFunc: func(c echo.Context, v middleware.RequestLoggerValues) error {
			var e *zerolog.Event
			if v.Error != nil {
				e = log.Error().
					Err(v.Error)
			} else {
				e = log.Trace()
			}
			e.
				Str("uri", v.URI).
				Int("status", v.Status).
				Str("latency", v.Latency.String()).
				Msg("request")
			return nil
		},
	}))

	e.StaticFS("/", shared.Must(fs.Sub(assets, "assets/index.html")))
	e.StaticFS("/client.wasm", shared.Must(fs.Sub(assets, "assets/generated/client.wasm")))
	e.StaticFS("/wasm_exec.js", shared.Must(fs.Sub(assets, "assets/generated/wasm_exec.js")))

	e.POST("/login", func(c echo.Context) error {
		var request shared.LoginRequest
		if err := c.Bind(&request); err != nil {
			return err
		}
		log.Debug().Str("username", request.Username).Str("password", request.Password).Msg("TODO request")
		return c.JSON(http.StatusUnauthorized, &shared.ErrorResponse{Message: "TODO testing"})
	})

	addr := "127.0.0.1:8000"
	go func() {
		if err := e.Start(addr); err != nil {
			log.Fatal().Err(err).Msg("server error")
		}
	}()
	log.Debug().Str("addr", addr).Msg("server started")
	select {}
}
