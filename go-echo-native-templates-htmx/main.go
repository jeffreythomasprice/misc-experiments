package main

import (
	"context"
	_ "embed"
	"errors"
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
		user, err := checkAuthToken(c, dbService)
		if errors.Is(err, errUnauthorized) {
			return views.NotLoggedInPage(c.Request().Context(), c.Response().Writer)
		}
		if err != nil {
			return err
		}
		return views.LoggedInPage(c.Request().Context(), c.Response().Writer, views.User{
			Username: user.Username,
			IsAdmin:  user.IsAdmin,
		})
	})

	e.POST("/login", func(c echo.Context) error {
		type Request struct {
			Username string `form:"username"`
			Password string `form:"password"`
		}

		log := zerolog.Ctx(c.Request().Context())

		var request Request
		if err := c.Bind(&request); err != nil {
			log.Debug().Err(err).Msg("error parsing request")
			return err
		}

		updatedLog := log.With().Str("username", request.Username).Logger()
		log = &updatedLog

		log.Trace().Msg("checking login status")
		user, err := dbService.CheckPassword(nil, request.Username, request.Password)
		if err != nil {
			return err
		}
		if user == nil {
			return views.ErrorsResponse(c.Request().Context(), c.Response().Writer, "Invalid username or password")
		}

		log.Trace().Msg("login successful")

		if err := createAuthToken(log, c, user); err != nil {
			return err
		}

		return views.LoggedInResponse(
			c.Request().Context(),
			c.Response().Writer,
			views.User{
				Username: user.Username,
				IsAdmin:  user.IsAdmin,
			},
		)
	})

	e.GET("/index.css", func(c echo.Context) error {
		return c.Blob(http.StatusOK, "text/css", indexCss)
	})

	if err := e.Start("127.0.0.1:8000"); err != nil {
		log.Fatal().Err(err)
	}
}
