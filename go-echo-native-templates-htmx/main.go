package main

import (
	"context"
	_ "embed"
	"experiment/db"
	"experiment/logging"
	"experiment/views"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

//go:embed static/index.css
var indexCss []byte

func main() {
	log := logging.ZerologInitPretty()

	e := echo.New()
	e.HideBanner = true
	e.Use(middleware.RequestID())
	logging.InitEcho(e, log)

	clicks := uint64(0)

	dbService, err := db.NewService(log.WithContext(context.Background()))
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create database")
	}
	// TODO testing
	{
		ok, err := dbService.CheckPassword("foo", "bar")
		log.Debug().Bool("ok", ok).Err(err).Msg("foo")
		ok, err = dbService.CheckPassword("admin", "admin")
		log.Debug().Bool("ok", ok).Err(err).Msg("admin")
	}

	e.GET("/", func(c echo.Context) error {
		return views.ClicksPage(c.Request().Context(), c.Response().Writer, clicks)
	})

	e.POST("/click", func(c echo.Context) error {
		clicks++
		return views.ClicksResponse(c.Request().Context(), c.Response().Writer, clicks)
	})

	e.GET("/index.css", func(c echo.Context) error {
		return c.Blob(http.StatusOK, "text/css", indexCss)
	})

	if err := e.Start("127.0.0.1:8000"); err != nil {
		log.Fatal().Err(err)
	}
}
