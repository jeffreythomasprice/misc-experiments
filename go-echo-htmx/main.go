package main

import (
	"database/sql"
	"errors"
	"experiment/db"
	"fmt"
	"html/template"
	"log/slog"
	"os"
	"path"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	_ "github.com/mattn/go-sqlite3"
	slogecho "github.com/samber/slog-echo"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		AddSource: true,
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

	e.POST("/login", func(c echo.Context) error {
		type request struct {
			Username string `form:"username"`
			Password string `form:"password"`
		}
		var req request
		if err := c.Bind(&req); err != nil {
			return t.RenderErrorMessage(c, "Missing parameters")
		}

		user, err := dbService.GetUserAndValidatePassword(req.Username, req.Password)
		if errors.Is(err, sql.ErrNoRows) || errors.Is(err, db.ErrBadPassword) {
			return t.RenderErrorMessage(c, "Invalid credentials")
		} else if err != nil {
			slog.Error("error looking up user to authenticate", "err", err)
			return t.RenderErrorMessage(c, "Error looking up user")
		}
		return t.RenderLoggedInPage(c, user)
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
