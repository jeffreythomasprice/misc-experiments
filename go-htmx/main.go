package main

import (
	"fmt"
	"net/http"
	"net/url"
	"os"
	"path"

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

	type MessageRequest struct {
		Message string
	}
	validateMessage := func(form url.Values) (*MessageRequest, error) {
		msg, err := ValidateSingleFormField(
			form,
			"message",
			MinStringLength(1),
		)
		if err != nil {
			return nil, err
		}
		return &MessageRequest{
			Message: msg,
		}, nil
	}
	r.Post(
		"/api/message",
		mustTemplateFunc[interface{}](`<li>
			TODO JEFF does this template do anything?
		</li>`).
			CreateHttpFormHandler(func(r *http.Request, form url.Values) (interface{}, error) {
				msg, err := validateMessage(form)
				if err != nil {
					return nil, err
				}
				slog.Info("TODO JEFF", "msg", msg)

				return nil, nil
			}),
	)

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
