package main

import (
	"embed"
	"log/slog"
	"net/http"
	"os"
	"shared"
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/components"
	. "github.com/maragudk/gomponents/html"
)

//go:embed assets/generated
var generatedAssets embed.FS

func main() {
	shared.InitSlog()

	e := echo.New()
	e.HideBanner = true
	e.HidePort = true
	e.Debug = true

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	e.GET("/", htmlNodeHandler(index))
	e.GET("/client.wasm", echo.StaticFileHandler("assets/generated/client.wasm", generatedAssets))
	e.GET("/wasm_exec.js", echo.StaticFileHandler("assets/generated/wasm_exec.js", generatedAssets))

	addr := "127.0.0.1:8000"
	slog.Info("listening", "addr", addr)
	e.Logger.Fatal(e.Start(addr))
}

func index(c echo.Context) (Node, error) {
	return HTML5(HTML5Props{
		Head: []Node{
			StyleEl(Text(`
				.error {
					font-weight: bold;
					color: red;
				}
			`)),
			Script(Src("/wasm_exec.js")),
			Script(Raw(`
				(async () => {
					const go = new Go();
					const wasm = await WebAssembly.instantiateStreaming(fetch("client.wasm"), go.importObject);
					go.run(wasm.instance);
				})()
					.catch(err => {
						console.error("error loading webassembly client", err);
					});
			`)),
		},
	}), nil
}

func htmlNodeHandler(f func(c echo.Context) (Node, error)) echo.HandlerFunc {
	return func(c echo.Context) error {
		node, err := f(c.(echo.Context))
		if err != nil {
			return err
		}
		var s strings.Builder
		if err := node.Render(&s); err != nil {
			return err
		}
		return c.HTML(http.StatusOK, s.String())
	}
}

func fail(msg string, err error) {
	slog.Error(msg, "err", err)
	os.Exit(1)
}
