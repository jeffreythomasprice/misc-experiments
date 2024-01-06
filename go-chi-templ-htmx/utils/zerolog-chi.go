package utils

import (
	"net/http"
	"runtime/debug"
	"time"

	"github.com/go-chi/chi/middleware"
	"github.com/rs/zerolog/log"
)

func ZerologMiddleware() func(http.Handler) http.Handler {
	// stolen from:
	// https://github.com/ironstar-io/chizerolog/blob/master/main.go

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// TODO needed?
			log := log.Logger.With().Logger()

			ww := middleware.NewWrapResponseWriter(w, r.ProtoMajor)

			start := time.Now()
			defer func() {
				end := time.Now()

				// Recover and record stack traces in case of a panic
				if rec := recover(); rec != nil {
					log.Error().
						Timestamp().
						Interface("Recover", rec).
						Bytes("Stack", debug.Stack()).
						Msg("request error")
					http.Error(ww, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
				}

				// log end request
				log.Info().
					Timestamp().
					Fields(map[string]interface{}{
						"RemoteAddr": r.RemoteAddr,
						"URL":        r.URL.Path,
						"Method":     r.Method,
						"Status":     ww.Status(),
						// TODO fancy duration
						"TimeMillis": float64(end.Sub(start).Nanoseconds()) / 1000000.0,
					}).
					Msg("request")
			}()

			next.ServeHTTP(ww, r)
		})
	}
}
