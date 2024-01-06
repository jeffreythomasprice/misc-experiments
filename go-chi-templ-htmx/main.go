package main

import (
	"embed"
	"io/fs"
	"net/http"
	"strconv"

	"github.com/go-chi/chi"
	"github.com/rs/zerolog/log"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"

	"experiment/db"

	//lint:ignore ST1001 don't worry about it
	. "experiment/utils"
)

type State struct {
	count    int
	messages []*db.Message
}

//go:embed web-files/*
var webFilesFS embed.FS

func main() {
	ZerologInitPretty()

	gormdb := Assert(gorm.Open(sqlite.Open("local.db"), &gorm.Config{}))
	gormdb.AutoMigrate(&db.Message{})

	router := chi.NewRouter()

	router.Use(ZerologMiddleware())

	state := &State{
		count: 0,
	}

	reloadMessages := func() {
		// TODO no assert
		state.messages = Assert(db.ListMessages(gormdb))
	}
	reloadMessages()

	router.Get("/", func(w http.ResponseWriter, r *http.Request) {
		RenderToHttpResponse(index(state), w)
	})

	router.Handle("/*", http.FileServer(http.FS(Assert(fs.Sub(webFilesFS, "web-files")))))

	router.Post("/click", func(w http.ResponseWriter, r *http.Request) {
		state.count++
		RenderToHttpResponse(countResults(state), w)
	})

	router.Post("/message", func(w http.ResponseWriter, r *http.Request) {
		if err := r.ParseForm(); err != nil {
			// TODO error responses
			log.Panic().Err(err).Msg("fatal")
		}

		message := r.FormValue("message")
		log.Info().Str("message", message).Msg("creating message")

		// TODO no assert
		result := Assert(db.CreateMessage(gormdb, &db.Message{Message: message}))
		state.messages = append(state.messages, result)
		go reloadMessages()

		RenderToHttpResponse(messages(state.messages), w)
	})

	router.Get("/message/{id}", func(w http.ResponseWriter, r *http.Request) {
		// TODO no assert
		id := uint(Assert(strconv.ParseUint(chi.URLParam(r, "id"), 10, 32)))
		log.Info().Uint("id", id).Msg("editing message")

		// TODO no assert
		message := Assert(db.GetMessage(gormdb, id))

		RenderToHttpResponse(messageRowEdit(message), w)
	})

	router.Put("/message/{id}", func(w http.ResponseWriter, r *http.Request) {
		// TODO no assert
		id := uint(Assert(strconv.ParseUint(chi.URLParam(r, "id"), 10, 32)))
		if err := r.ParseForm(); err != nil {
			// TODO error responses
			log.Panic().Err(err).Msg("fatal")
		}
		message := r.FormValue("message")
		log.Info().Uint("id", id).Str("message", message).Msg("editing message")

		// TODO no assert
		result := Assert(db.UpdateMessage(gormdb, &db.Message{ID: id, Message: message}))

		go reloadMessages()
		RenderToHttpResponse(messageRow(result), w)
	})

	router.Delete("/message/{id}", func(w http.ResponseWriter, r *http.Request) {
		// TODO no assert
		id := uint(Assert(strconv.ParseUint(chi.URLParam(r, "id"), 10, 32)))
		log.Info().Uint("id", id).Msg("deleting message")

		if err := db.DeleteMessage(gormdb, id); err != nil {
			log.Err(err).Uint("id", id).Msg("error deleting")
			// TODO respond with error
		}

		go reloadMessages()
	})

	addr := "127.0.0.1:8000"
	log.Info().Str("addr", addr).Msg("listening")
	if err := http.ListenAndServe(addr, router); err != nil {
		log.Panic().Err(err).Msg("fatal")
	}
}
