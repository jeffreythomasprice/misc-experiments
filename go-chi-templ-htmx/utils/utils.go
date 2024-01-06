package utils

import "github.com/rs/zerolog/log"

func Assert[T any](value T, err error) T {
	if err != nil {
		log.Panic().Err(err).Msg("assert failed")
	}
	return value
}
