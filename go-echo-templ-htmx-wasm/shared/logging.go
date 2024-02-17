package shared

import (
	"os"
	"runtime"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func InitLogger() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	log.Logger = log.Output(zerolog.ConsoleWriter{
		Out:        os.Stdout,
		NoColor:    runtime.GOOS == "js",
		TimeFormat: "2006-01-02T15:04:05.999Z07:00",
	}).
		Level(zerolog.DebugLevel)
}
