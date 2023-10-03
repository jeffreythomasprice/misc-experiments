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
		GET: htmlPageHandler(func(w io.Writer) error {
			return index(
				w,
				// TODO testing headers, remove m
				map[string]string{
					"foo": "bar",
					"baz": "asdf",
				},
				func(w io.Writer) error {
					return templates.ExecuteTemplate(w, "hello", nil)
				},
			)
		}),
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

type HtmlPageHandlerFunc func(w io.Writer) error

func index(w io.Writer, headers map[string]string, content func(w io.Writer) error) error {
	var headerStr string
	if headers != nil {
		b, err := json.Marshal(headers)
		if err != nil {
			return fmt.Errorf("error marshalling headers to json for index: %w", err)
		}
		headerStr = string(b)
	}

	var contentStr strings.Builder
	if err := content(&contentStr); err != nil {
		return fmt.Errorf("error rendering content for index: %w", err)
	}

	return templates.ExecuteTemplate(w, "index", map[string]any{
		"bodyAttributes": map[string]any{
			"headers": template.HTMLAttr(headerStr),
		},
		"content": template.HTML(contentStr.String()),
	})
}

func htmlPageHandler(f HtmlPageHandlerFunc) http.HandlerFunc {
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
