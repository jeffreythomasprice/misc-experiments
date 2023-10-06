package main

import (
	"fmt"
	"log/slog"
	"net/http"
	"os"

	"github.com/a-h/templ"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level:     slog.LevelDebug,
		AddSource: true,
	})))

	http.HandleFunc("/", httpMethods{
		GET: templ.Handler(index(clickForm())).ServeHTTP,
	}.Handler())

	clicks := 0
	http.HandleFunc("/click", httpMethods{
		POST: func(w http.ResponseWriter, r *http.Request) {
			clicks++
			templ.Handler(clickContent(fmt.Sprintf("%v", clicks))).ServeHTTP(w, r)
		},
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
