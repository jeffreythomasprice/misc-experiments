package main

import (
	"embed"
	"experiment/auth"
	"fmt"
	"io/fs"
	"net/http"
	"os"

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

	authRoutes := e.Group("", auth.RequireAuth(func(c echo.Context, err error) error {
		// TODO handle different errors
		log.Warn().Err(err).Msg("redirecting to login")
		return htmxRedirect(c, "/login")
	}))

	e.GET(
		"/static/*",
		echo.WrapHandler(
			http.StripPrefix(
				"/static/",
				http.FileServer(http.FS(staticFilesFS)),
			),
		),
	)

	e.GET("/login", func(c echo.Context) error {
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

		// parse
		if err := c.Bind(&request); err != nil {
			return respondWithError(fmt.Sprintf("Bad request: %v", err))
		}
		log.Debug().Str("username", request.Username).Msg("login request")

		// validate
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
			if err := auth.NewCookie(c, &auth.NewJwtRequest{
				Username: request.Username,
			}); err != nil {
				return err
			}
			return htmxRedirect(c, "/")
		} else {
			return respondWithError("Invalid credentials")
		}
	})

	e.POST("/logout", func(c echo.Context) error {
		auth.ClearCookie(c)
		return htmxRedirect(c, "/login")
	})

	authRoutes.GET("/", func(c echo.Context) error {
		return templCompToEchoCtx(c, index(func() templ.Component {
			return testContent(auth.Get(c).Username)
		}))
	})

	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

func templCompToEchoCtx(c echo.Context, comp templ.Component) error {
	return comp.Render(c.Request().Context(), c.Response().Writer)
}
