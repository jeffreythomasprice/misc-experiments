package main

import (
	"bytes"
	"embed"
	"fmt"
	"html/template"
	"io"
	"io/fs"
	"log/slog"
	"net/http"
	"os"
	"shared"
	"strings"
)

//go:embed assets/generated
var generatedAssets embed.FS

//go:embed assets/templates
var templateAssets embed.FS

func main() {
	shared.InitSlog()

	templates, err := template.ParseFS(templateAssets, "assets/templates/**")
	if err != nil {
		fail("failed to parse templates", err)
	}

	index := serveTemplate(templates, "index", nil)
	http.Handle("/", GET(index))
	http.Handle("/index.html", GET(index))
	http.Handle("/client.wasm", GET(serveFSFile(generatedAssets, "assets/generated/client.wasm")))
	http.Handle("/wasm_exec.js", GET(serveFSFile(generatedAssets, "assets/generated/wasm_exec.js")))

	addr := "127.0.0.1:8000"
	go func() {
		if err := http.ListenAndServe(addr, nil); err != nil {
			fail("server error", err)
		}
	}()
	slog.Info("server started", "addr", addr)
	select {}
}

func GET(h http.Handler) http.Handler {
	return HTTPMethodSwitch{GET: h}.Handler()
}

type HTTPMethodSwitch struct {
	GET, PUT, POST, DELETE http.Handler
}

func (x HTTPMethodSwitch) Handler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		h, ok := map[string]http.Handler{
			"GET":    x.GET,
			"PUT":    x.PUT,
			"POST":   x.POST,
			"DELETE": x.DELETE,
		}[r.Method]
		if !ok || h == nil {
			http.Error(w, http.StatusText(http.StatusMethodNotAllowed), http.StatusMethodNotAllowed)
			return
		}
		h.ServeHTTP(w, r)
	}
}

func serveTemplate(t *template.Template, name string, data any) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var s strings.Builder
		if err := t.ExecuteTemplate(&s, name, data); err != nil {
			slog.Error("error rendering template", "name", name, "data", data)
			http.Error(w, fmt.Sprintf("failed to render template %v", name), http.StatusInternalServerError)
			return
		}
		if _, err := fmt.Fprint(w, s.String()); err != nil {
			slog.Error("error writing template content to http response", "err", err)
		}
	}
}

func serveFSFile(filesystem fs.FS, path string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		f, err := filesystem.Open(path)
		if err != nil {
			slog.Error("error opening file", "path", path, "err", err)
			http.Error(w, fmt.Sprintf("failed to open file %v", path), http.StatusInternalServerError)
			return
		}
		stat, err := f.Stat()
		if err != nil {
			slog.Error("error getting stat for file", "path", path, "err", err)
			http.Error(w, fmt.Sprintf("failed to stat file %v", path), http.StatusInternalServerError)
			return
		}
		fReadSeeker, ok := f.(io.ReadSeeker)
		if ok {
			http.ServeContent(w, r, path, stat.ModTime(), fReadSeeker)
			return
		}
		b, err := io.ReadAll(f)
		if err != nil {
			slog.Error("trying to read file in full so that we have a seekable version, but failed to read", "path", path, "err", err)
			http.Error(w, fmt.Sprintf("failed to read file %v", path), http.StatusInternalServerError)
		}
		http.ServeContent(w, r, path, stat.ModTime(), bytes.NewReader(b))
	}
}

func fail(msg string, err error) {
	slog.Error(msg, "err", err)
	os.Exit(1)
}
