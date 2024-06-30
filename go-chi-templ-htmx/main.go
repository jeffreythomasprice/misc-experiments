package main

import (
	"database/sql"
	_ "embed"
	"fmt"
	"net/http"
	"os"
	"path"

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

	var db *sqlx.DB
	{
		driver := "sqlite3"
		connectionStr := fmt.Sprintf("file:%s/experiment.db", dataDir)
		log := log.With().
			Str("driver", driver).
			Str("connection", connectionStr).
			Logger()
		log.Debug().Msg("connecting to database")
		db, err = sqlx.Connect(driver, connectionStr)
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

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		index(
			func() templ.Component {
				return loginForm()
			},
		).Render(r.Context(), w)
	})

	r.Get("/login", func(w http.ResponseWriter, r *http.Request) {
		// TODO deduplicate
		index(
			func() templ.Component {
				return loginForm()
			},
		).Render(r.Context(), w)
	})

	r.Get("/createUser", func(w http.ResponseWriter, r *http.Request) {
		index(
			func() templ.Component {
				return createUserForm()
			},
		).Render(r.Context(), w)
	})

	r.Post("/login", func(w http.ResponseWriter, r *http.Request) {
		if err := r.ParseForm(); err != nil {
			log.Panic().Err(err).Msg("failed to parse form")
		}
		username := r.Form.Get("username")
		password := r.Form.Get("password")
		log.Trace().
			Str("username", username).
			Str("password", password).
			Msg("login")
		panic("TODO not implemented")
	})

	r.Post("/createUser", func(w http.ResponseWriter, r *http.Request) {
		if err := r.ParseForm(); err != nil {
			log.Panic().Err(err).Msg("failed to parse form")
		}
		username := r.Form.Get("username")
		password := r.Form.Get("password")
		confirmPassword := r.Form.Get("confirmPassword")
		log.Trace().
			Str("username", username).
			Msg("createUser")
		if len(username) < 1 {
			// TODO error
		}
		if len(password) < 1 {
			// TODO error
		}
		if password != confirmPassword {
			// TODO error
		}
		if err = createUser(db, &User{
			Username: username,
			Password: sql.NullString{String: password, Valid: true},
		}); err != nil {
			// TODO error
		}
		// TODO return login form
		panic("TODO not implemented")
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
