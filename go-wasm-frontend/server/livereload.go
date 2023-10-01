package main

import (
	"fmt"
	"log/slog"
	"net/http"
	"time"

	"github.com/go-chi/chi"
	"github.com/olahol/melody"
)

func liveReload(mux *chi.Mux, pattern string) {
	m := melody.New()
	startupTime := time.Now().UnixMilli()
	m.HandleConnect(func(s *melody.Session) {
		text := fmt.Sprintf("%v", startupTime)
		if err := s.Write([]byte(text)); err != nil {
			slog.Error("error writing to websocket", "err", err)
		}
	})
	mux.HandleFunc("/_liveReload", func(w http.ResponseWriter, r *http.Request) {
		if err := m.HandleRequest(w, r); err != nil {
			slog.Error("error responding to websocket", "err", err)
		}
	})
}
