package main

import (
	"embed"
	"errors"
	"fmt"
	"io/fs"
	"log/slog"
	"net/http"
	"os"
	"shared"
	"sync"
	"time"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/olahol/melody"
)

//go:embed embed
var assets embed.FS

func main() {
	shared.InitSlog()

	r := chi.NewRouter()
	r.Use(middleware.Logger)

	assets := http.FS(assets)
	r.Get("/", serveFile(assets, "embed/index.html"))
	r.Get("/wasm_exec.js", serveFile(assets, "embed/wasm_exec.js"))
	r.Get("/client.wasm", serveFile(assets, "embed/client.wasm"))

	r.HandleFunc("/_liveReload", liveReloadWebSocketHandler())

	var wg sync.WaitGroup
	wg.Add(1)
	host := "localhost:8000"
	go func() {
		defer wg.Done()
		if err := http.ListenAndServe(host, r); err != nil {
			slog.Error("http server error", "err", err)
		}
	}()
	slog.Debug("listening", "host", host)

	wg.Wait()
	slog.Debug("done")
}

func liveReloadWebSocketHandler() http.HandlerFunc {
	m := melody.New()
	response := []byte(fmt.Sprintf("%d", time.Now().UTC().UnixMilli()))
	m.HandleConnect(func(s *melody.Session) {
		s.Write(response)
	})
	return func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	}
}

func sfatal(message string, err error) {
	slog.Error(message, "err", err)
	os.Exit(1)
}

func serveFile(fileSystem http.FileSystem, path string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// http.ServeContent
		file, err := fileSystem.Open(path)
		if err != nil {
			slog.Error("error opening file", "path", path, "err", err)
			if errors.Is(err, fs.ErrNotExist) {
				respondError(w, http.StatusNotFound)
				return
			}
			respondError(w, http.StatusInternalServerError)
			return
		}
		stat, err := file.Stat()
		if err != nil {
			slog.Error("error getting file stat", "path", path, "err", err)
			respondError(w, http.StatusInternalServerError)
			return
		}
		http.ServeContent(w, r, path, stat.ModTime(), file)
		if err := file.Close(); err != nil {
			slog.Error("error closing file", "path", path, "err", err)
		}
	}
}

func respondError(w http.ResponseWriter, code int) {
	http.Error(w, http.StatusText(code), code)
}
