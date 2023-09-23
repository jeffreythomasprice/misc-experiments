package main

import (
	"embed"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"log/slog"
	"net/http"
	"shared"
	"slices"
	"sync"
	"time"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/google/uuid"
	"github.com/olahol/melody"
)

//go:embed embed
var assets embed.FS

func main() {
	shared.InitSlog()

	r := chi.NewRouter()
	r.Use(middleware.Logger)

	assets := http.FS(assets)
	r.Get("/", serveFile(assets, "embed/index.html"))
	r.Get("/wasm_exec.js", serveFile(assets, "embed/wasm_exec.js"))
	r.Get("/client.wasm", serveFile(assets, "embed/client.wasm"))

	r.HandleFunc("/_liveReload", liveReloadWebSocketHandler())

	r.HandleFunc("/login", newJsonHandlerFunc[shared.WebsocketLoginRequest, shared.WebsocketLoginResponse](func(request *shared.WebsocketLoginRequest) (*shared.WebsocketLoginResponse, error) {
		id := uuid.NewString()
		slog.Info("new client", "id", id, "name", request.Name)
		return &shared.WebsocketLoginResponse{ID: id}, nil
	}))

	r.HandleFunc("/ws", chatWebsocketHandler())

	var wg sync.WaitGroup
	wg.Add(1)
	host := "localhost:8000"
	go func() {
		defer wg.Done()
		if err := http.ListenAndServe(host, r); err != nil {
			slog.Error("http server error", "err", err)
		}
	}()
	slog.Debug("listening", "host", host)

	wg.Wait()
	slog.Debug("done")
}

func liveReloadWebSocketHandler() http.HandlerFunc {
	m := melody.New()
	response := []byte(fmt.Sprintf("%d", time.Now().UTC().UnixMilli()))
	m.HandleConnect(func(s *melody.Session) {
		s.Write(response)
	})
	return func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	}
}

func chatWebsocketHandler() http.HandlerFunc {
	m := melody.New()

	verifiedSessions := make([]*melody.Session, 0)
	var verifiedSessionsMutex sync.Mutex

	getID := func(s *melody.Session) (string, bool) {
		value, ok := s.Get("id")
		if ok {
			return value.(string), true
		} else {
			return "", false
		}
	}

	setID := func(s *melody.Session, id string) {
		s.Set("id", id)
	}

	closeSession := func(s *melody.Session) {
		if err := s.Close(); err != nil {
			slog.Error("error when closing session", "err", err)
		}
	}

	m.HandleMessage(func(s *melody.Session, b []byte) {
		var message shared.WebsocketClientToServerMessage
		if err := json.Unmarshal(b, &message); err != nil {
			slog.Error("error unmarshalling websocket message", "err", err)
			return
		}

		switch message.Type {
		case shared.WebsocketClientToServerMessageTypeLogin:
			slog.Debug(
				"received message",
				"type", message.Type,
				"id", message.Login.ID,
			)
			// if session is already verified, close it
			id, ok := getID(s)
			if ok {
				slog.Error(
					"session is already verified",
					"existing id", id,
					"incoming id", message.Login.ID,
				)
				closeSession(s)
				return
			}
			// this session is now verified
			setID(s, message.Login.ID)
			verifiedSessionsMutex.Lock()
			verifiedSessions = append(verifiedSessions, s)
			verifiedSessionsMutex.Unlock()

		case shared.WebsocketClientToServerMessageTypeSend:
			slog.Debug(
				"received message",
				"type", message.Type,
				"message", message.Send.Message,
			)
			id, ok := getID(s)
			// if session is unverified, close it
			if !ok {
				slog.Error("message received before verification", id)
				closeSession(s)
				return
			}
			// broadcast response to all connected sessions
			outgoingBytes, err := json.Marshal(&shared.WebsocketServerToClientMessage{
				Type: shared.WebsocketServerToClientMessageTypeSend,
				Send: &shared.WebsocketServerToClientMessageSend{
					SenderID: id,
					Message:  message.Send.Message,
				},
			})
			if err != nil {
				slog.Error("failed to serialize message", "err", err)
				return
			}
			verifiedSessionsMutex.Lock()
			cloneOfVerifiedSessions := slices.Clone(verifiedSessions)
			verifiedSessionsMutex.Unlock()
			for _, s := range cloneOfVerifiedSessions {
				if err := s.Write(outgoingBytes); err != nil {
					slog.Error("error writing to session", "session", s, "err", err)
					closeSession(s)
				}
			}

		default:
			slog.Error("unrecognized message type", "type", message.Type)
		}
	})

	return func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	}
}

func serveFile(fileSystem http.FileSystem, path string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// http.ServeContent
		file, err := fileSystem.Open(path)
		if err != nil {
			slog.Error("error opening file", "path", path, "err", err)
			if errors.Is(err, fs.ErrNotExist) {
				respondError(w, http.StatusNotFound)
				return
			}
			respondError(w, http.StatusInternalServerError)
			return
		}
		stat, err := file.Stat()
		if err != nil {
			slog.Error("error getting file stat", "path", path, "err", err)
			respondError(w, http.StatusInternalServerError)
			return
		}
		http.ServeContent(w, r, path, stat.ModTime(), file)
		if err := file.Close(); err != nil {
			slog.Error("error closing file", "path", path, "err", err)
		}
	}
}

func newJsonHandlerFunc[RequestType, ResponseType any](f func(request *RequestType) (*ResponseType, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// TODO logging should have request content stuff like remote addr

		// TODO assert that it's got the application/json or */* header
		// strings.Split(r.Header.Get("accept"), ",")

		// TODO assert that content type is application/json
		// r.Header.Get("content-type")

		request, err := shared.UnmarshalJson[RequestType](r.Body)
		if err != nil {
			slog.Error("error reading request body", "err", err)
			respondError(w, 400)
			return
		}
		slog.Debug("request", "body", request)

		response, err := f(request)
		if err != nil {
			slog.Error("error handling json request", "err", err)
			respondError(w, 500)
			return
		}
		slog.Debug("response", "body", response)

		responseBytes, err := json.Marshal(response)
		if err != nil {
			slog.Error("error marshalling response body", "err", err)
			respondError(w, 500)
			return
		}

		w.Header().Add("content-type", "application/json")
		w.WriteHeader(200)
		_, err = w.Write(responseBytes)
		if err != nil {
			slog.Error("error writing response to http request", "err", err)
		}
	}
}

func respondError(w http.ResponseWriter, code int) {
	http.Error(w, http.StatusText(code), code)
}
