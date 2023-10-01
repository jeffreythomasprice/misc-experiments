package main

import (
	"embed"
	"errors"
	"fmt"
	"io/fs"
	"log/slog"
	"net/http"
	"shared"
	"strings"
	"sync"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/components"
	. "github.com/maragudk/gomponents/html"
)

//go:embed embed
var assets embed.FS

func main() {
	shared.InitSlog()

	r := chi.NewRouter()
	r.Use(middleware.RequestLogger(&SlogLogFormatter{}))

	assets := http.FS(assets)
	r.Get("/", newHtmlHandlerFunc(func(w http.ResponseWriter, r *http.Request) ([]Node, error) {
		return []Node{index()}, nil
	}))
	r.Get("/wasm_exec.js", serveFile(assets, "embed/wasm_exec.js"))
	r.Get("/client.wasm", serveFile(assets, "embed/client.wasm"))

	liveReload(r, "/_liveReload")

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

func index() Node {
	return HTML5(HTML5Props{
		Title:    "experiment",
		Language: "en",
		Head: []Node{
			Script(Src("wasm_exec.js")),
			Script(Raw(`
				(async () => {
					const go = new Go();
					const wasm = await WebAssembly.instantiateStreaming(fetch("client.wasm"), go.importObject);
					go.run(wasm.instance);
				})()
					.catch(err => {
						console.error("failed to load wasm client", err);
					});
			`)),
		},
	})
}

func newHtmlHandlerFunc(f func(w http.ResponseWriter, r *http.Request) ([]Node, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		nodes, err := f(w, r)
		if err != nil {
			writeErrorResponse(w, err, 500, "internal server error")
			return
		}

		var s strings.Builder
		for _, child := range nodes {
			if err := child.Render(&s); err != nil {
				writeErrorResponse(w, err, 500, "internal server error")
				return
			}
		}

		w.Header().Add("content-type", "text/html")
		_, err = fmt.Fprint(w, s.String())
		if err != nil {
			slog.Error("error writing content to http writer", "err", err)
		}
	}
}

func serveFile(fileSystem http.FileSystem, path string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		file, err := fileSystem.Open(path)
		if err != nil {
			slog.Error("error opening file", "path", path, "err", err)
			if errors.Is(err, fs.ErrNotExist) {
				writeErrorResponse(w, err, http.StatusNotFound, "not found")
				return
			}
			writeErrorResponse(w, err, http.StatusInternalServerError, "internal server error")
			return
		}
		stat, err := file.Stat()
		if err != nil {
			writeErrorResponse(w, err, http.StatusInternalServerError, "internal server error")
			return
		}
		http.ServeContent(w, r, path, stat.ModTime(), file)
		if err := file.Close(); err != nil {
			slog.Error("error closing file", "path", path, "err", err)
		}
	}
}

func writeErrorResponse(w http.ResponseWriter, err error, statusCode int, message string) {
	slog.Error(
		"error response",
		"err", err,
		"statusCode", statusCode,
	)
	w.WriteHeader(statusCode)
	_, err = fmt.Fprint(w, message)
	if err != nil {
		slog.Error(
			"an error occurred writing an error message in response to a previous error",
			"err", err,
			"statusCode", statusCode,
		)
	}
}
