package main

import (
	_ "embed"
	"net/http"
	"os"
	"strconv"

	"github.com/Lavalier/zchi"
	"github.com/a-h/templ"
	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

//go:embed static/vendor/htmx.min.js
var htmxJs []byte

//go:embed static/index.css
var indexCss []byte

func main() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	zerolog.SetGlobalLevel(zerolog.TraceLevel)
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: "2006-01-02T15:04:05.000Z"})

	r := chi.NewRouter()

	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Recoverer)
	r.Use(zchi.Logger(log.Logger))

	r.Get("/htmx.min.js", serveStaticBytes("text/javascript", htmxJs))
	r.Get("/index.css", serveStaticBytes("text/css", indexCss))

	clicks := 0

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		clicksStr := strconv.FormatInt(int64(clicks), 10)
		index(
			func() templ.Component {
				return clickComp(clicksStr)
			},
		).Render(r.Context(), w)
	})

	r.Post("/click", func(w http.ResponseWriter, r *http.Request) {
		clicks++
		clicksStr := strconv.FormatInt(int64(clicks), 10)
		clickResultsComp(clicksStr).Render(r.Context(), w)
	})

	address := "127.0.0.1:8000"
	log.Info().Str("address", address).Msg("server started")
	http.ListenAndServe(address, r)
}

func serveStaticBytes(contentType string, data []byte) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Add("Content-Type", contentType)
		w.Write(data)
	}
}
