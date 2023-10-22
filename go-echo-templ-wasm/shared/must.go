package shared

import (
	"github.com/rs/zerolog/log"
)

func Must[T any](result T, err error) T {
	return Must1[T](result, err)
}

func Must0(err error) {
	if err != nil {
		log.Fatal().Err(err).Msg("must failed")
	}
}

func Must1[T any](result T, err error) T {
	if err != nil {
		log.Fatal().Err(err).Msg("must failed")
	}
	return result
}
