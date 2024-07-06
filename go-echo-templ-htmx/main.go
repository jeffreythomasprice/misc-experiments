package main

import (
	"embed"
	"fmt"
	"io/fs"
	"net/http"
	"os"
	"time"

	"github.com/a-h/templ"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/ziflex/lecho/v3"
)

type LoginRequest struct {
	Username string `form:"username"`
	Password string `form:"password"`
}

//go:embed static/*
var staticFiles embed.FS

func main() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	zerologLogInstance := zerolog.New(zerolog.ConsoleWriter{
		Out:        os.Stderr,
		TimeFormat: "2006-01-02T15:04:05.999Z07:00",
	}).
		With().
		Timestamp().
		Logger()
	log.Logger = zerologLogInstance

	staticFilesFS, err := fs.Sub(staticFiles, "static")
	if err != nil {
		log.Fatal().Err(err).Send()
	}

	e := echo.New()
	lechoLogInstance := lecho.From(zerologLogInstance)
	e.Logger = lechoLogInstance
	e.Use(middleware.RequestID())
	e.Use(lecho.Middleware(lecho.Config{
		Logger: lechoLogInstance,
	}))
	e.HideBanner = true

	e.Pre(func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if err := next(c); err != nil {
				return err
			}

			// TODO deduplicate cookie name
			auth, err := c.Cookie("auth")
			if err != nil {
				return err
			}
			if auth != nil {
				log.Debug().Str("auth", auth.Value).Msg("TODO auth cookie")
			} else {
				log.Debug().Msg("TODO no auth cookie")
			}

			return nil
		}
	})

	e.GET(
		"/static/*",
		echo.WrapHandler(
			http.StripPrefix(
				"/static/",
				http.FileServer(http.FS(staticFilesFS)),
			),
		),
	)

	e.GET("/", func(c echo.Context) error {
		return templCompToEchoCtx(c, index(func() templ.Component {
			return loginForm(LoginRequest{}, nil)
		}))
	})

	e.POST("/login", func(c echo.Context) error {
		var request LoginRequest

		respondWithError := func(messages ...string) error {
			log.Warn().Strs("errorMessages", messages).Msg("login request failed")
			return templCompToEchoCtx(c, loginForm(request, messages))
		}

		if err := c.Bind(&request); err != nil {
			return respondWithError(fmt.Sprintf("Bad request: %v", err))
		}
		log.Debug().Str("username", request.Username).Msg("login request")

		errorMessages := make([]string, 0)
		if len(request.Username) == 0 {
			errorMessages = append(errorMessages, "Username is required")
		}
		if len(request.Password) == 0 {
			errorMessages = append(errorMessages, "Password is required")
		}
		if len(errorMessages) > 0 {
			return respondWithError(errorMessages...)
		}

		// TODO use real db here
		if request.Username == "admin" && request.Password == "admin" {
			c.SetCookie(&http.Cookie{
				// TODO deduplicate cookie name
				Name:    "auth",
				Value:   "TODO jwt here",
				Expires: time.Now().Add(24 * time.Hour),
			})
			// TODO redirect with set cookie?
			return templCompToEchoCtx(c, index(func() templ.Component {
				return testContent()
			}))
		} else {
			return respondWithError("Invalid credentials")
		}
	})

	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

func templCompToEchoCtx(ctx echo.Context, c templ.Component) error {
	return c.Render(ctx.Request().Context(), ctx.Response().Writer)
}
