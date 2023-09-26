package main

import (
	"encoding/json"
	"experiments/livereload"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"sync"

	"github.com/go-chi/chi"
	"github.com/google/uuid"
	g "github.com/maragudk/gomponents"
	c "github.com/maragudk/gomponents/components"
	h "github.com/maragudk/gomponents/html"
	"github.com/olahol/melody"
)

type client struct {
	Name, ID string
	Session  *melody.Session
}

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level: slog.LevelDebug,
		},
	)))

	r := chi.NewRouter()

	liveReloadScript, err := livereload.Script("/_liveReloadFoobar")
	if err != nil {
		panic(err)
	}
	r.HandleFunc("/_liveReloadFoobar", livereload.HandlerFunc(r))

	clients := make(map[string]*client)

	r.Get("/", pageHandlerFunc(
		func() ([]g.Node, error) {
			return []g.Node{liveReloadScript}, nil
		},
		func() ([]g.Node, error) {
			return []g.Node{
				h.Div(
					h.ID("root"),
					h.Div(g.Text("Enter a name")),
					h.FormEl(
						g.Attr("hx-post", "/login"),
						h.Input(
							h.Name("name"),
							h.AutoFocus(),
						),
					),
				),
			}, nil
		},
	))

	r.Post("/login", nodeHandlerFunc(func(r *http.Request) ([]g.Node, error) {
		if err := r.ParseForm(); err != nil {
			return nil, fmt.Errorf("error reading request body: %w", err)
		}

		name := r.Form.Get("name")
		id := uuid.NewString()
		slog.Info(
			"logging in",
			"name", name,
			"id", id,
		)

		clients[id] = &client{
			Name:    name,
			ID:      id,
			Session: nil,
		}

		return []g.Node{
			h.Div(
				h.ID("root"),
				g.Attr("hx-swap-oob", "innerHTML"),
				h.Div(g.Textf("Name: %v", name)),
				h.Div(g.Textf("ID: %v", id)),
				h.Div(
					g.Attr("hx-ext", "ws"),
					g.Attr("ws-connect", "/ws"),
					websocketFormLogin(id),
					h.Div(h.ID("ws-messages")),
				),
			),
		}, nil
	}))

	m := melody.New()

	m.HandleConnect(func(s *melody.Session) {
		slog.Debug(
			"ws connected",
			"remote addr", s.RemoteAddr().String(),
		)
	})

	m.HandleMessage(func(s *melody.Session, b []byte) {
		slog.Debug("received message", "msg", string(b))

		disconnectAndRespondWithError := func() {
			// TODO respond with error to ui
			if err := s.Close(); err != nil {
				slog.Error("error closing websocket in response to a previous error", "err", err)
			}
		}

		var msg struct {
			Type    string `json:"type"`
			ID      string `json:"id"`
			Message string `type:"message"`
		}
		if err := json.Unmarshal(b, &msg); err != nil {
			slog.Error("error unmarshalling json to check type", "err", err)
			disconnectAndRespondWithError()
			return
		}

		client, ok := clients[msg.ID]
		if !ok {
			slog.Error("no such client", "id", msg.ID)
			disconnectAndRespondWithError()
			return
		}

		response := []g.Node{
			websocketForm(msg.ID),
		}

		switch msg.Type {
		case "login":
			if client.Session != nil {
				slog.Error("client is already initialized", "id", msg.ID)
				disconnectAndRespondWithError()
				return
			}
			client.Session = s
			slog.Info("client is now initialized", "id", msg.ID)

		case "send":
			slog.Debug("message received", "id", msg.ID, "msg", msg.Message)
			response = append(response, websocketMessageNode(fmt.Sprintf("server responding to: %v from %v", msg.Message, client.Name)))

		default:
			slog.Error("unrecognized message type", "type", msg.Type)
			disconnectAndRespondWithError()
			return
		}

		writeNodesToWebSocket(
			s,
			response...,
		)
	})

	r.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	})

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

func websocketFormPreConnect() g.Node {
	return h.FormEl(
		h.ID("ws-form"),
	)
}

func websocketFormLogin(id string) g.Node {
	return h.FormEl(
		h.ID("ws-form"),
		g.Attr("ws-send"),
		g.Attr("hx-trigger", "revealed"),
		h.Input(
			h.Type("hidden"),
			h.Name("type"),
			h.Value("login"),
		),
		h.Input(
			h.Type("hidden"),
			h.Name("id"),
			h.Value(id),
		),
	)
}

func websocketForm(id string) g.Node {
	return h.FormEl(
		h.ID("ws-form"),
		g.Attr("ws-send"),
		h.Input(
			h.Type("hidden"),
			h.Name("type"),
			h.Value("send"),
		),
		h.Input(
			h.Type("hidden"),
			h.Name("id"),
			h.Value(id),
		),
		h.Input(
			h.Name("message"),
			h.AutoFocus(),
		),
	)
}

func websocketMessageNode(message string) g.Node {
	return h.Div(
		h.ID("ws-messages"),
		g.Attr("hx-swap-oob", "beforeend"),
		h.Div(
			g.Text(message),
		),
	)
}

func pageHandlerFunc(head, body func() ([]g.Node, error)) http.HandlerFunc {
	return nodeHandlerFunc(func(r *http.Request) ([]g.Node, error) {
		renderedHead, err := head()
		if err != nil {
			return nil, fmt.Errorf("error rendering page head: %w", err)
		}
		renderedBody, err := body()
		if err != nil {
			return nil, fmt.Errorf("error rendering page body: %w", err)
		}
		return []g.Node{
			c.HTML5(c.HTML5Props{
				Title:    "Experiment",
				Language: "en",
				Head: append(
					[]g.Node{
						h.Script(h.Src("https://unpkg.com/htmx.org@1.9.5")),
						h.Script(h.Src("https://unpkg.com/htmx.org@1.9.5/dist/ext/ws.js")),
						h.Script(g.Text(`
							htmx.logAll();
						`)),
					},
					renderedHead...,
				),
				Body: renderedBody,
			}),
		}, nil
	})
}

func nodeHandlerFunc(f func(r *http.Request) ([]g.Node, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		nodes, err := f(r)
		if err != nil {
			slog.Error("error rendering nodes", "err", err)
			// TODO respond with an error message
			return
		}
		for _, node := range nodes {
			if err := node.Render(w); err != nil {
				slog.Error("error rendering node", "err", err)
				// TODO respond with an error message
				return
			}
		}
	}
}

func writeNodesToWebSocket(s *melody.Session, nodes ...g.Node) error {
	var w strings.Builder
	for _, n := range nodes {
		if err := n.Render(&w); err != nil {
			return err
		}
	}
	return s.Write([]byte(w.String()))
}
