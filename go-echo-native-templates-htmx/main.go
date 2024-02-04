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
	"github.com/rs/zerolog"
)

//go:embed static/index.css
var indexCss []byte

func main() {
	log := logging.ZerologInitPretty()

	e := echo.New()
	e.HideBanner = true
	e.Use(middleware.RequestID())
	logging.InitEcho(e, log)

	dbService, err := db.NewService(log.WithContext(context.Background()))
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create database")
	}

	e.GET("/", func(c echo.Context) error {
		return views.NotLoggedInPage(c.Request().Context(), c.Response().Writer)
	})

	e.POST("/login", func(c echo.Context) error {
		type Request struct {
			Username string `form:"username"`
			Password string `form:"password"`
		}
		var request Request
		if err := c.Bind(&request); err != nil {
			return err
		}
		log := zerolog.Ctx(c.Request().Context())
		log.Trace().Str("username", request.Username).Msg("checking login status")
		ok, err := dbService.CheckPassword(request.Username, request.Password)
		if err != nil {
			return err
		}
		if ok {
			return views.LoggedInResponse(
				c.Request().Context(),
				c.Response().Writer,
				views.User{
					// TODO don't use request, pull actual data including isAdmin when checking password
					Username: request.Username,
				},
			)
		} else {
			return views.ErrorsResponse(c.Request().Context(), c.Response().Writer, "Invalid username or password")
		}
	})

	e.GET("/index.css", func(c echo.Context) error {
		return c.Blob(http.StatusOK, "text/css", indexCss)
	})

	if err := e.Start("127.0.0.1:8000"); err != nil {
		log.Fatal().Err(err)
	}
}
