package main

import (
	_ "embed"
	"fmt"
	"net/http"
	"os"
	"path"
	"strconv"

	"github.com/Lavalier/zchi"
	"github.com/a-h/templ"
	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
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

	dataDir, err := dataDir()
	if err != nil {
		log.Fatal().Err(err).Msg("failed to find data dir")
	}

	{
		driver := "sqlite3"
		connectionStr := fmt.Sprintf("file:%s/experiment.db", dataDir)
		log := log.With().
			Str("driver", driver).
			Str("connection", connectionStr).
			Logger()
		log.Debug().Msg("connecting to database")
		db, err := sqlx.Connect(driver, connectionStr)
		if err != nil {
			log.Fatal().Err(err).Msg("failed to open database")
		}
		err = migrate(db)
		if err != nil {
			log.Fatal().Err(err).Msg("failed to execute database migration")
		}
	}

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

func dataDir() (string, error) {
	exePath, err := os.Executable()
	if err != nil {
		return "", fmt.Errorf("failed to find executable path: %w", err)
	}
	result := path.Dir(exePath)
	log.Trace().
		Str("exePath", exePath).
		Str("dataDir", result).
		Msg("")
	return result, nil
}
