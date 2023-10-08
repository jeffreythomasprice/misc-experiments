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
	"github.com/olahol/melody"
	slogecho "github.com/samber/slog-echo"

	_ "github.com/mattn/go-sqlite3"
)

const jwtCookieName = "auth"

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

	db, err := openDatabase()
	if err != nil {
		return err
	}
	defer func() {
		if err := db.Close(); err != nil {
			slog.Error("error closing database", "err", err)
		}
	}()

	propertiesService, err := NewPropertiesService(db)
	if err != nil {
		return err
	}

	jwtService, err := NewJWTService(propertiesService)
	if err != nil {
		return err
	}

	usersService, err := NewUsersService(db)
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
	e.GET("/index.css", echo.StaticFileHandler("assets/index.css", assets))
	e.GET("/client.wasm", echo.StaticFileHandler("assets/generated/client.wasm", assets))
	e.GET("/wasm_exec.js", echo.StaticFileHandler("assets/generated/wasm_exec.js", assets))

	e.GET("/checkToken", func(c echo.Context) error {
		user, response, err := checkAuth(c, jwtService, usersService)
		if user == nil || err != nil {
			slog.Debug("user is not authenticated", "user", user, "err", err)
			return err
		}
		slog.Debug("user is authenticated", "user", user)
		return c.JSON(http.StatusOK, response)
	})

	e.POST("/login", func(c echo.Context) error {
		var request shared.LoginRequest
		if err := c.Bind(&request); err != nil {
			return err
		}

		user, err := usersService.ValidatePassword(request.Username, request.Password)
		if err != nil {
			slog.Error("error validating user's password", "err", err)
			return statusCodeError(c, http.StatusInternalServerError)
		}
		if user == nil {
			return statusCodeError(c, http.StatusUnauthorized)
		}

		token, claims, err := jwtService.Create(shared.JWTCustomClaims{
			Username: user.Username,
			IsAdmin:  user.IsAdmin,
		})
		if err != nil {
			slog.Error("error making jwt", "err", err)
			return statusCodeError(c, http.StatusInternalServerError)
		}

		addAuthCookie(c, token, time.Unix(claims.ExpiresAt, 0))

		return c.JSON(
			http.StatusOK,
			&shared.LoginResponse{
				Token: token,
			},
		)
	})

	e.POST("/logout", func(c echo.Context) error {
		addAuthCookie(c, "", time.Now())

		return c.NoContent(http.StatusOK)
	})

	m := melody.New()
	m.HandleConnect(func(s *melody.Session) {
		slog.Debug("TODO websocket connected", "remote addr", s.RemoteAddr())

		s.Write([]byte("TODO text message from server"))
		s.WriteBinary([]byte("TODO binary message from server"))
	})
	m.HandleClose(func(s *melody.Session, code int, message string) error {
		slog.Debug("TODO websocket disconnected", "remote addr", s.RemoteAddr())
		return nil
	})
	m.HandleMessage(func(s *melody.Session, b []byte) {
		slog.Debug("TODO websocket text message", "remote addr", s.RemoteAddr(), "msg", string(b))
	})
	m.HandleMessageBinary(func(s *melody.Session, b []byte) {
		slog.Debug("TODO websocket binary message", "remote addr", s.RemoteAddr(), "msg", string(b))
	})
	e.GET("/ws", func(c echo.Context) error {
		return m.HandleRequest(c.Response(), c.Request())
	})

	addr := "127.0.0.1:8000"
	slog.Info("listening", "addr", addr)
	return e.Start(addr)
}

func addAuthCookie(c echo.Context, value string, expires time.Time) {
	cookie := &http.Cookie{}
	cookie.Name = jwtCookieName
	cookie.Value = value
	cookie.Expires = expires
	c.SetCookie(cookie)
}

func checkAuth(c echo.Context, jwtService *JWTService, usersService *UsersService) (*User, *shared.CheckTokenResponse, error) {
	cookie, err := c.Cookie(jwtCookieName)
	if err != nil {
		if errors.Is(err, http.ErrNoCookie) {
			slog.Debug("no auth cookie, not authenticated")
			return nil, nil, statusCodeError(c, http.StatusUnauthorized)
		}
		slog.Error("error checking auth cookie", "err", err)
		return nil, nil, statusCodeError(c, http.StatusInternalServerError)
	}

	token := cookie.Value
	claims, err := jwtService.Validate(token)
	if err != nil {
		slog.Error("validation failed on jwt", "token", token, "err", err)
		return nil, nil, statusCodeError(c, http.StatusUnauthorized)
	}

	user, err := usersService.GetByName(claims.Username)
	if err != nil {
		slog.Error("error finding user by name", "username", claims.Username, "err", err)
		return nil, nil, statusCodeError(c, http.StatusInternalServerError)
	}
	if user == nil {
		return nil, nil, statusCodeError(c, http.StatusUnauthorized)
	}
	return user, &shared.CheckTokenResponse{Token: token}, nil
}

func statusCodeError(c echo.Context, code int) error {
	return c.JSON(
		code,
		&shared.ErrorResponse{
			Message: http.StatusText(code),
		},
	)
}
