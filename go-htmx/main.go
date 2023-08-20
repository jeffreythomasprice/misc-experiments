package main

import (
	"fmt"
	"html/template"
	"io"
	"net/http"
	"os"
	"path"
	"strings"

	"log/slog"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level:     slog.LevelDebug,
			AddSource: false,
		},
	)))

	r := chi.NewRouter()

	r.Use(middleware.RequestLogger(&SlogLogFormatter{}))

	// static files
	webDir, err := getWebDir()
	if err != nil {
		slog.Error("error getting current directory", "err", err)
		os.Exit(1)
	}
	r.Handle("/*", http.FileServer(http.Dir(webDir)))

	count := 0
	testTemplate := mustTemplateFunc[int]("<div>{{ . }}</div>")
	r.HandleFunc("/api/test", testTemplate.CreateHttpHandler(func() (int, error) {
		// time.Sleep(time.Second * 5)
		result := count
		count += 1
		return result, nil
	}))

	addr := "127.0.0.1"
	port := 8000
	slog.Info("listening on", "addr", addr, "port", port)
	if err := http.ListenAndServe(fmt.Sprintf("%v:%v", addr, port), r); err != nil {
		slog.Error("server error", "err", err)
	}
}

func getWebDir() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	return path.Join(cwd, "web"), nil
}

type TemplateFunc[T any] func(data T) (string, error)

func newTemplateFunc[T any](text string) (TemplateFunc[T], error) {
	t, err := template.New("").Parse(text)
	if err != nil {
		return nil, err
	}
	return func(data T) (string, error) {
		var s strings.Builder
		if err := t.Execute(&s, data); err != nil {
			return "", err
		}
		return s.String(), nil
	}, nil
}

func mustTemplateFunc[T any](text string) TemplateFunc[T] {
	result, err := newTemplateFunc[T](text)
	if err != nil {
		panic(err)
	}
	return result
}

func (f TemplateFunc[T]) WriteTo(w io.Writer, data T) error {
	text, err := f(data)
	if err != nil {
		return err
	}
	_, err = fmt.Fprint(w, text)
	return err
}

func (f TemplateFunc[T]) CreateHttpHandler(dataFactory func() (T, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		data, err := dataFactory()
		if err != nil {
			slog.Error("failed to generate data to render", "err", err)
			w.WriteHeader(500)
			return
		}

		if err := f.WriteTo(w, data); err != nil {
			slog.Error("failed to render template", "err", err)
			w.WriteHeader(500)
			return
		}
	}
}
