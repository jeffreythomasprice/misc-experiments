package main

import (
	_ "embed"
	"net/http"
	"shared"
	"strings"

	"github.com/a-h/templ"
	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog/log"
)

//go:embed static/index.css
var indexCss []byte

//go:embed static/vendor/htmx/htmx.min.js
var htmxMinJs []byte

//go:embed generated/client.wasm
var clientWasm []byte

//go:embed generated/wasm_exec.js
var wasmExecJs []byte

func main() {
	shared.InitLogger()

	e := echo.New()
	e.HideBanner = true

	e.GET("/index.css", staticFileHandler(indexCss, "text/css"))
	e.GET("/htmx.min.js", staticFileHandler(htmxMinJs, "application/javascript"))
	e.GET("/client.wasm", staticFileHandler(clientWasm, "application/wasm"))
	e.GET("/wasm_exec.js", staticFileHandler(wasmExecJs, "application/javascript"))

	clicks := 0

	e.GET("/", templHandler(func(ctx echo.Context) (templ.Component, error) {
		return Index(ClickForm(clicks)), nil
	}))

	e.POST("/click", templHandler(func(ctx echo.Context) (templ.Component, error) {
		clicks++
		return ClickResults(clicks), nil
	}))

	e.GET(shared.ReloadPath, ReloadHandler())

	if err := e.Start("127.0.0.1:8000"); err != nil {
		log.Fatal().Err(err).Msg("server error")
	}
}

func templHandler(f func(echo.Context) (templ.Component, error)) echo.HandlerFunc {
	return func(c echo.Context) error {
		comp, err := f(c)
		if err != nil {
			return err
		}
		var s strings.Builder
		if err := comp.Render(c.Request().Context(), &s); err != nil {
			return err
		}
		return c.HTML(http.StatusOK, s.String())
	}
}

func staticFileHandler(b []byte, contentType string) echo.HandlerFunc {
	return func(c echo.Context) error {
		c.Response().Header().Set(echo.HeaderContentType, contentType)
		c.Response().Writer.WriteHeader(http.StatusOK)
		_, err := c.Response().Writer.Write(b)
		return err
	}
}
