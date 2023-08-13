package main

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"path"
	"shared/demo"
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

	// static files
	binDir, err := getDirectoryRunningProcessIsIn()
	if err != nil {
		slog.Error("error getting current directory", "err", err)
		os.Exit(1)
	}
	r.Handle("/*", http.FileServer(http.Dir(path.Join(binDir, "web"))))

	r.HandleFunc("/ws/autoreload", reload.NewAutoReloadServerHandlerFunc())

	// TODO JEFF json websocket connection needs a wrapper for going from json.RawMessage to tagged union, same on client
	websocketServer, websocketHandlerFunc := websockets.NewWebsocketServerHandlerFunc()
	r.HandleFunc("/ws", websocketHandlerFunc)
	go func() {
		for connection := range websocketServer.Incoming() {
			jsonConnection := websockets.NewJsonWebsocketConnection[json.RawMessage](connection)
			go func() {
				for rawMessage := range jsonConnection.Incoming() {
					message, err := demo.MessageTaggedUnion.Unmarshall(rawMessage)
					if err != nil {
						slog.Error("error unmarshalling message", "err", err)
						continue
					}
					switch t := message.(type) {
					case *demo.ClientMessage:
						panic("TODO JEFF handle this message type")
					case *demo.ClientSetName:
						panic("TODO JEFF handle this message type")
					default:
						slog.Debug("unhandled message type", "type", t)
					}
				}
			}()
		}
	}()

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
