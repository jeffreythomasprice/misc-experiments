package main

import (
	"embed"
	"log/slog"
	"net/http"
	"shared"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"
)

//go:embed assets
var assets embed.FS

func main() {
	shared.InitSlog()

	e := echo.New()
	e.HideBanner = true
	e.HidePort = true
	e.Debug = true

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	e.GET("/", echo.StaticFileHandler("assets/index.html", assets))
	e.GET("/client.wasm", echo.StaticFileHandler("assets/generated/client.wasm", assets))
	e.GET("/wasm_exec.js", echo.StaticFileHandler("assets/generated/wasm_exec.js", assets))

	e.POST("/login", func(c echo.Context) error {
		var request shared.LoginRequest
		if err := c.Bind(&request); err != nil {
			return err
		}
		slog.Debug("TODO got login request", "request", request)

		c.JSON(
			http.StatusOK,
			&shared.LoginResponse{
				Token: "TODO jwt here",
			},
		)

		return nil
	})

	addr := "127.0.0.1:8000"
	slog.Info("listening", "addr", addr)
	e.Logger.Fatal(e.Start(addr))
}
