package main

import (
	"context"
	_ "embed"
	"errors"
	"experiment/auth"
	"experiment/db"
	"experiment/logging"
	"experiment/views"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/rs/zerolog"
	"golang.org/x/net/websocket"
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
		_, user, err := auth.CheckToken(nil, c, dbService)
		if errors.Is(err, auth.ErrUnauthorized) {
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

		if err := auth.CreateToken(log, c, user); err != nil {
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

	e.POST("/logout", func(c echo.Context) error {
		log, _, err := auth.CheckToken(nil, c, dbService)
		if errors.Is(err, auth.ErrUnauthorized) {
			return views.NotLoggedInPage(c.Request().Context(), c.Response().Writer)
		}
		if err != nil {
			return err
		}
		log.Info().Msg("logging out")
		auth.ClearToken(c)
		return views.NotLoggedInPage(c.Request().Context(), c.Response().Writer)
	})

	e.GET("/ws", func(c echo.Context) error {
		log, _, err := auth.CheckToken(nil, c, dbService)
		if err != nil {
			return err
		}

		websocket.Handler(func(ws *websocket.Conn) {
			ctx := c.Request().Context()

			defer ws.Close()

			go func() {
				// TODO why do we have to sleep?
				time.Sleep(time.Second * 1)

				var buf strings.Builder
				if err := views.WebsocketMessage(ctx, &buf, "TODO testing"); err != nil {
					log.Error().Err(err).Msg("error writing websocket message to buffer")
					return
				}
				if err := websocket.Message.Send(ws, buf.String()); err != nil {
					log.Error().Err(err).Msg("error sending to websocket")
				}
			}()

			for {
				type message struct {
					Message string `json:"wsMessage"`
				}
				var msg message
				err := websocket.JSON.Receive(ws, &msg)
				if errors.Is(err, io.EOF) {
					return
				}
				if err != nil {
					log.Error().Err(err).Msg("error receiving from websocket")
					continue
				}
				log.Trace().Str("payload", msg.Message).Msg("received message from websocket")
			}
		}).
			ServeHTTP(c.Response(), c.Request())
		return nil
	})

	e.GET("/index.css", func(c echo.Context) error {
		return c.Blob(http.StatusOK, "text/css", indexCss)
	})

	if err := e.Start("127.0.0.1:8000"); err != nil {
		log.Fatal().Err(err)
	}
}
