package main

import (
	"database/sql"
	"errors"
	"experiment/db"
	"fmt"
	"html/template"
	"log/slog"
	"net/http"
	"os"
	"path"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	_ "github.com/mattn/go-sqlite3"
	slogecho "github.com/samber/slog-echo"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		AddSource: false,
		Level:     slog.LevelDebug,
	})))

	dbService, err := db.NewService(getTempDir)
	if err != nil {
		fatal("failed to init db", err)
	}
	defer dbService.Close()

	e := echo.New()

	t := &assetTemplates{template.Must(template.ParseFS(embedAssets, "assets/*"))}
	e.Renderer = t

	e.HideBanner = true
	e.HidePort = true
	e.Debug = true

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	e.FileFS("/styles.css", "assets/styles.css", embedAssets)

	e.GET("/", func(c echo.Context) error {
		return t.RenderLoginPage(c)
	})

	e.GET("/loggedIn", func(c echo.Context) error {
		cookie, err := c.Cookie("auth")

		// not authenticated, missing cookie
		if errors.Is(err, http.ErrNoCookie) {
			// TODO include an error message?
			return c.Redirect(http.StatusSeeOther, "/")
		}

		// actual error
		if err != nil {
			slog.Error("error looking for auth cookie", "err", err)
			// TODO include an error message?
			return c.Redirect(http.StatusSeeOther, "/")
		}

		// TODO check cookie value for validity

		slog.Debug("TODO JEFF logged in", "cookie", cookie)

		panic("TODO need to look up user from cookie and render logged in page")
		// return t.RenderLoggedInPage(c, ?)
	})

	e.POST("/login", func(c echo.Context) error {
		type request struct {
			Username string `form:"username"`
			Password string `form:"password"`
		}
		var req request
		if err := c.Bind(&req); err != nil {
			return t.RenderErrorMessage(c, "Missing parameters")
		}

		_, err := dbService.GetUserAndValidatePassword(req.Username, req.Password)
		if errors.Is(err, sql.ErrNoRows) || errors.Is(err, db.ErrBadPassword) {
			return t.RenderErrorMessage(c, "Invalid credentials")
		} else if err != nil {
			slog.Error("error looking up user to authenticate", "err", err)
			return t.RenderErrorMessage(c, "Error looking up user")
		}

		// c.SetCookie(&http.Cookie{
		// 	Name:  "auth",
		// 	Value: "TODO jwt here",
		// 	// TODO use jwt expiration
		// 	Expires: time.Now().Add(time.Second * 15),
		// })

		return c.Redirect(http.StatusSeeOther, "/loggedIn")
	})

	addr := "127.0.0.1:8000"
	slog.Info("server started", "addr", addr)
	if err := e.Start(addr); err != nil {
		fatal("server error", err)
	}
}

func getTempDir() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", fmt.Errorf("failed to get current working directory: %w", err)
	}
	tmpDir := path.Join(cwd, "bin")
	slog.Debug("paths", "cwd", cwd, "tmpDir", tmpDir)
	if err := os.MkdirAll(tmpDir, os.ModePerm); err != nil {
		return "", fmt.Errorf("failed to create temp dir: %w", err)
	}
	return tmpDir, nil
}

func fatal(msg string, err error) {
	slog.Error(msg, "err", err)
	os.Exit(1)
}
