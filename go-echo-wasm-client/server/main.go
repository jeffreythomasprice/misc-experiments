package main

import (
	"embed"
	"errors"
	"log/slog"
	"net/http"
	"os"
	"shared"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"

	_ "github.com/mattn/go-sqlite3"
)

type user struct {
	token  string
	claims *shared.JWTClaims
}

//go:embed assets
var assets embed.FS

func main() {
	if err := run(); err != nil {
		slog.Error("fatal", "err", err)
		os.Exit(1)
	}
}

func run() error {
	shared.InitSlog()

	db, err := NewDB()
	if err != nil {
		return err
	}
	defer db.Close()

	jwtService, err := NewJWTService(db)
	if err != nil {
		return err
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
		user, err := checkAuth(c, jwtService)
		if user == nil || err != nil {
			slog.Debug("user is not authenticated", "user", user, "err", err)
			return err
		}
		slog.Debug("user is authenticated", "user", user)
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

		if request.Password != "password" {
			return statusCodeError(c, http.StatusUnauthorized)
		}

		token, claims, err := jwtService.Create(shared.JWTCustomClaims{
			Username: request.Username,
		})
		if err != nil {
			slog.Error("error making jwt", "err", err)
			return statusCodeError(c, http.StatusInternalServerError)
		}

		authCookie := &http.Cookie{}
		authCookie.Name = "auth"
		authCookie.Value = token
		authCookie.Domain = "*"
		authCookie.Expires = time.Unix(claims.ExpiresAt, 0)
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
	return e.Start(addr)
}

func checkAuth(c echo.Context, jwtService *JWTService) (*user, error) {
	authCookie, err := c.Cookie("auth")
	if err != nil {
		if errors.Is(err, http.ErrNoCookie) {
			slog.Debug("no auth cookie, not authenticated")
			return nil, statusCodeError(c, http.StatusUnauthorized)
		}
		slog.Error("error checking auth cookie", "err", err)
		return nil, statusCodeError(c, http.StatusInternalServerError)
	}
	token := authCookie.Value
	claims, err := jwtService.Validate(token)
	if err != nil {
		slog.Error("validation failed on jwt", "token", token, "err", err)
		return nil, statusCodeError(c, http.StatusUnauthorized)
	}
	return &user{
		token:  token,
		claims: claims,
	}, nil
}

func statusCodeError(c echo.Context, code int) error {
	return c.JSON(
		code,
		&shared.ErrorResponse{
			Message: http.StatusText(code),
		},
	)
}
