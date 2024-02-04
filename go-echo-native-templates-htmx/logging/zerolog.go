package logging

import (
	"os"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/ziflex/lecho/v3"
)

const RFC3339Milli = "2006-01-02T15:04:05.999Z07:00"

func zerologInitCommon() {
	zerolog.TimeFieldFormat = time.RFC3339Nano
}

func ZerologInitPretty() zerolog.Logger {
	zerologInitCommon()
	log.Logger = log.Output(zerolog.ConsoleWriter{
		Out:        os.Stdout,
		TimeFormat: RFC3339Milli,
	})
	return log.Logger
}

func ZerologInitJson() zerolog.Logger {
	zerologInitCommon()
	zerolog.TimeFieldFormat = RFC3339Milli
	log.Logger = zerolog.New(os.Stdout).
		With().
		Timestamp().
		Logger()
	return log.Logger
}

func InitEcho(e *echo.Echo, log zerolog.Logger) {
	echoLog := lecho.From(log)
	e.Logger = echoLog
	e.Use(lecho.Middleware(lecho.Config{
		Logger: echoLog,
	}))
}
