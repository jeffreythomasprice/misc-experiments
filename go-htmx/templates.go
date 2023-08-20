package main

import (
	"fmt"
	"html/template"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strings"
)

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

func (f TemplateFunc[T]) CreateHttpHandler(dataFactory func(r *http.Request) (T, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var data T
		var err error

		if dataFactory != nil {
			data, err = dataFactory(r)
			if err != nil {
				slog.Error("failed to generate data to render", "err", err)
				w.WriteHeader(GetStatusCodeForError(err))
				return
			}
		}

		if err := f.WriteTo(w, data); err != nil {
			slog.Error("failed to render template", "err", err)
			w.WriteHeader(GetStatusCodeForError(err))
			return
		}
	}
}

func (f TemplateFunc[T]) CreateHttpFormHandler(dataFactory func(r *http.Request, form url.Values) (T, error)) http.HandlerFunc {
	return f.CreateHttpHandler(func(r *http.Request) (T, error) {
		if err := r.ParseForm(); err != nil {
			var result T
			return result, NewHttpError(400, fmt.Sprintf("failed to parse form data: %v", err))
		}
		return dataFactory(r, r.Form)
	})
}
