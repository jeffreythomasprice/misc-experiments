package main

import (
	"embed"
	"errors"
	"log/slog"
	"net/http"
	"shared"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"

	_ "github.com/mattn/go-sqlite3"
)

type user struct {
	token string
}

//go:embed assets
var assets embed.FS

func main() {
	shared.InitSlog()

	// TODO testing
	db, err := NewDB()
	if err != nil {
		panic(err)
	}
	defer db.Close()
	{
		value, err := db.GetProperty("test")
		slog.Debug("TODO JEFF get prop", "value", value, "err", err)
		if err != nil {
			panic(err)
		}
		err = db.SetProperty("test", "foo")
		if err != nil {
			panic(err)
		}
		value, err = db.GetProperty("test")
		slog.Debug("TODO JEFF get prop", "value", value, "err", err)
		if err != nil {
			panic(err)
		}
	}

	e := echo.New()
	e.HideBanner = true
	e.HidePort = true
	e.Debug = true

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	e.GET("/", echo.StaticFileHandler("assets/index.html", assets))
	e.GET("/client.wasm", echo.StaticFileHandler("assets/generated/client.wasm", assets))
	e.GET("/wasm_exec.js", echo.StaticFileHandler("assets/generated/wasm_exec.js", assets))

	e.GET("/checkToken", func(c echo.Context) error {
		user, err := checkAuth(c)
		if user == nil || err != nil {
			return err
		}
		return c.JSON(
			http.StatusOK,
			&shared.LoginResponse{
				Token: user.token,
			},
		)
	})

	e.POST("/login", func(c echo.Context) error {
		var request shared.LoginRequest
		if err := c.Bind(&request); err != nil {
			return err
		}

		token := "TODO jwt here"

		authCookie := &http.Cookie{}
		authCookie.Name = "auth"
		authCookie.Value = token
		authCookie.Domain = "*"
		// TODO expires when jwt expires
		c.SetCookie(authCookie)

		return c.JSON(
			http.StatusOK,
			&shared.LoginResponse{
				Token: token,
			},
		)
	})

	addr := "127.0.0.1:8000"
	slog.Info("listening", "addr", addr)
	e.Logger.Fatal(e.Start(addr))
}

func checkAuth(c echo.Context) (*user, error) {
	authCookie, err := c.Cookie("auth")
	if err != nil {
		if errors.Is(err, http.ErrNoCookie) {
			return nil, statusCodeError(c, http.StatusUnauthorized)
		}
		slog.Error("error checking auth cookie", "err", err)
		return nil, statusCodeError(c, http.StatusInternalServerError)
	}
	return &user{authCookie.Value}, nil
}

func statusCodeError(c echo.Context, code int) error {
	return c.JSON(
		code,
		&shared.ErrorResponse{
			Message: http.StatusText(code),
		},
	)
}
