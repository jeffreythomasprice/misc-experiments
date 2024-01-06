package main

import (
	"net/http"

	"github.com/go-chi/chi"
	"github.com/rs/zerolog/log"

	//lint:ignore ST1001 don't worry about it
	. "experiment/utils"
)

type State struct {
	count int
}

func main() {
	router := chi.NewRouter()

	ZerologInitPretty()
	router.Use(ZerologMiddleware())

	state := &State{count: 0}

	router.Get("/", func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(Assert(RenderToString(index(state)))))
	})

	router.Post("/click", func(w http.ResponseWriter, r *http.Request) {
		state.count++
		w.Write([]byte(Assert(RenderToString(countResults(state)))))
	})

	addr := "127.0.0.1:8000"
	log.Info().Str("addr", addr).Msg("listening")
	if err := http.ListenAndServe(addr, router); err != nil {
		log.Panic().Err(err).Msg("fatal")
	}
}
