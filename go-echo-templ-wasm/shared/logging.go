package shared

import (
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func InitLogging(color bool) {
	zerolog.SetGlobalLevel(zerolog.TraceLevel)

	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs

	writer := zerolog.NewConsoleWriter()
	writer.NoColor = !color
	writer.TimeFormat = "2006-01-02T15:04:05.999Z07:00"

	log.Logger = zerolog.New(writer).
		With().
		Timestamp().
		Logger()
}
