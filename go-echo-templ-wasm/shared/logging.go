package shared

import "github.com/rs/zerolog"

func InitLogging() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	zerolog.SetGlobalLevel(zerolog.TraceLevel)
}
