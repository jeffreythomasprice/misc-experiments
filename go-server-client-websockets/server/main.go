package main

import (
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"path"
	"shared"
	"shared/websockets"
	"shared/websockets/reload"

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

	binDir, err := getDirectoryRunningProcessIsIn()
	if err != nil {
		panic(err)
	}
	r.Handle("/*", http.FileServer(http.Dir(path.Join(binDir, "web"))))

	// TODO JEFF demo
	websocketServer, websocketHandlerFunc := websockets.NewWebsocketServerHandlerFunc()
	r.HandleFunc("/ws", websocketHandlerFunc)
	go func() {
		for connection := range websocketServer.Incoming() {
			jsonConnection := websockets.NewJsonWebsocketConnection[shared.Message](connection)
			go func() {
				for message := range jsonConnection.Incoming() {
					slog.Info("incoming message", "text", message.Message)
				}
			}()
			jsonConnection.Send(shared.Message{
				Message: "Hello from server",
			})
		}
	}()

	r.HandleFunc("/ws/autoreload", reload.NewAutoReloadServerHandlerFunc())

	addr := "127.0.0.1"
	port := 8000
	slog.Info("listening on", "addr", addr, "port", port)
	if err := http.ListenAndServe(fmt.Sprintf("%v:%v", addr, port), r); err != nil {
		slog.Error("server error", "err", err)
	}
}

func getDirectoryRunningProcessIsIn() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	return path.Dir(path.Join(cwd, os.Args[0])), nil
}
