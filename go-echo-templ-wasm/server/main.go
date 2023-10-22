package main

import (
	"embed"
	"io/fs"
	"shared"

	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog/log"
)

//go:embed assets
var assets embed.FS

func main() {
	shared.InitLogging()

	e := echo.New()

	e.StaticFS("/", shared.Must(fs.Sub(assets, "assets/index.html")))
	e.StaticFS("/client.wasm", shared.Must(fs.Sub(assets, "assets/generated/client.wasm")))
	e.StaticFS("/wasm_exec.js", shared.Must(fs.Sub(assets, "assets/generated/wasm_exec.js")))

	addr := "127.0.0.1:8000"
	go func() {
		if err := e.Start(addr); err != nil {
			log.Fatal().Err(err).Msg("server error")
		}
	}()
	log.Debug().Str("addr", addr).Msg("server started")
	select {}
}
