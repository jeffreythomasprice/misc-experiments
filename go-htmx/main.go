package main

import (
	"embed"
	"encoding/json"
	"fmt"
	"html/template"
	"io"
	"log/slog"
	"net/http"
	"os"
	"strings"
)

type TemplateDataFunc[T any] func() (T, error)

type StringContentFunc func(w io.Writer) error

//go:embed templates/*
var templateFiles embed.FS

var templates *template.Template

func init() {
	templates = template.Must(template.ParseFS(templateFiles, "templates/*"))
}

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level:     slog.LevelDebug,
		AddSource: true,
	})))

	http.HandleFunc("/", httpMethods{
		GET: htmlPageHandler(newTemplateHandler[any](templates, "index", func() (any, error) {
			headers := map[string]string{
				"foo": "bar",
				"baz": "asdf",
			}

			var contentStr strings.Builder
			if err := templates.ExecuteTemplate(&contentStr, "hello", nil); err != nil {
				return nil, err
			}

			return index(headers, contentStr.String())
		})),
	}.Handler())

	// TODO testing, remove me
	clicks := 0
	http.HandleFunc("/click", httpMethods{
		PUT: htmlPageHandler(func(w io.Writer) error {
			clicks++
			return templates.ExecuteTemplate(w, "clickResult", clicks)
		}),
	}.Handler())

	addr := "127.0.0.1:8000"
	go func() {
		if err := http.ListenAndServe(addr, nil); err != nil {
			slog.Error("http listen error", "err", err)
			os.Exit(1)
		}
	}()
	slog.Debug("server started", "addr", addr)
	select {}
}

func index(headers map[string]string, content string) (any, error) {
	var headerStr string
	if headers != nil {
		b, err := json.Marshal(headers)
		if err != nil {
			return nil, fmt.Errorf("error marshalling headers to json for index: %w", err)
		}
		headerStr = string(b)
	}

	return map[string]any{
		"bodyAttributes": map[string]any{
			"headers": template.HTMLAttr(headerStr),
		},
		"content": template.HTML(content),
	}, nil
}

func newTemplateHandler[T any](t *template.Template, name string, data TemplateDataFunc[T]) StringContentFunc {
	return func(w io.Writer) error {
		d, err := data()
		if err != nil {
			return fmt.Errorf("error generating data for template rendering: %w", err)
		}
		var s strings.Builder
		if err := templates.ExecuteTemplate(&s, name, d); err != nil {
			return fmt.Errorf("error rendering template: %w", err)
		}
		if _, err := fmt.Fprint(w, s.String()); err != nil {
			return fmt.Errorf("error writing template content: %w", err)
		}
		return nil
	}
}

func htmlPageHandler(f StringContentFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var s strings.Builder
		if err := f(&s); err != nil {
			slog.Error("error rendering html page", "err", err)
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
		w.Header().Set("content-type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, err := fmt.Fprint(w, strings.TrimSpace(s.String()))
		if err != nil {
			slog.Error("error writing body to html page response", "err", err)
		}
	}
}

type httpMethods struct {
	GET, PUT, POST, DELETE http.HandlerFunc
}

func (m httpMethods) Handler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		f := map[string]http.HandlerFunc{
			http.MethodGet:    m.GET,
			http.MethodPost:   m.POST,
			http.MethodPut:    m.PUT,
			http.MethodDelete: m.DELETE,
		}[r.Method]
		if f == nil {
			w.WriteHeader(http.StatusMethodNotAllowed)
			return
		}
		f(w, r)
	}
}
