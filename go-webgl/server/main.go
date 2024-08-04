package main

import (
	"embed"
	"fmt"
	"io/fs"
	"os"
	"path"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/ziflex/lecho/v3"
)

//go:embed static/*
var staticFilesFS embed.FS

//go:embed generated/*
var generatedFilesFS embed.FS

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

	if err := changeWorkingDirToExecutableDir(); err != nil {
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

	{
		fs, err := fs.Sub(staticFilesFS, "static")
		if err != nil {
			log.Panic().Err(err).Send()
		}
		e.StaticFS("/", fs)
		e.StaticFS("/index.html", fs)
		e.StaticFS("/static/*", fs)
	}
	{
		fs, err := fs.Sub(generatedFilesFS, "generated")
		if err != nil {
			log.Panic().Err(err).Send()
		}
		e.StaticFS("/generated/*", fs)
	}

	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

func changeWorkingDirToExecutableDir() error {
	exe, err := os.Executable()
	if err != nil {
		return fmt.Errorf("failed to get exe location: %w", err)
	}

	wd, err := os.Getwd()
	if err != nil {
		return fmt.Errorf("failed to get current working dir: %w", err)
	}

	desired := path.Dir(exe)

	log.Debug().
		Str("exe", exe).
		Str("workingDir", wd).
		Str("desired", desired).
		Msg("changing current working dir")

	err = os.Chdir(desired)
	if err != nil {
		return fmt.Errorf("failed to change working dir: %w", err)
	}
	return nil
}
