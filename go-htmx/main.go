package main

import (
	"encoding/json"
	"experiments/livereload"
	"fmt"
	"html/template"
	"log/slog"
	"net/http"
	"os"
	"reflect"
	"strings"
	"sync"

	"github.com/go-chi/chi"
	"github.com/google/uuid"
	g "github.com/maragudk/gomponents"
	c "github.com/maragudk/gomponents/components"
	h "github.com/maragudk/gomponents/html"
	"github.com/olahol/melody"
)

type Client struct {
	Name, ID string
	Session  *melody.Session
}

type WebsocketLogin struct {
	ID string `json:"id"`
}

type WebsocketMessage struct {
	Message string             `json:"message"`
	ID      string             `json:"id"`
	Headers map[string]*string `json:"HEADERS"`
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

	clients := make(map[string]*Client)

	r.Get("/", pageHandlerFunc(
		func() ([]g.Node, error) {
			return []g.Node{liveReloadScript}, nil
		},
		func() ([]g.Node, error) {
			return []g.Node{
				h.Div(g.Text("Enter a name")),
				h.FormEl(
					g.Attr("hx-post", "/login"),
					h.Input(
						h.Name("name"),
						h.AutoFocus(),
					),
				),

				// TODO JEFF no

				// h.Button(
				// 	g.Text("Click Me"),
				// 	g.Attr("hx-post", "/clicked1"),
				// 	g.Attr("hx-target", "#target1"),
				// ),
				// h.Div(h.ID("target1")),

				// h.Button(
				// 	g.Text("Also Click Me"),
				// 	g.Attr("hx-post", "/clicked2"),
				// 	g.Attr("hx-target", "#target2"),
				// ),
				// h.Div(h.ID("target2")),

				// h.Div(
				// 	g.Attr("hx-ws", "connect:/ws"),
				// 	websocketForm(),
				// 	h.Div(
				// 		h.ID("ws-messages"),
				// 	),
				// ),
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

		clients[id] = &Client{
			Name:    name,
			ID:      id,
			Session: nil,
		}

		return []g.Node{
			h.Div(g.Textf("Name: %v", name)),
			h.Div(g.Textf("ID: %v", id)),
			h.Div(
				g.Attr("hx-ws", "connect:/ws"),
				g.Attr("hx-trigger", "every 1s"),
				websocketFormLogin(id),
				h.Div(h.ID("ws-messages")),
			),
		}, nil
	}))

	// TODO JEFF no

	// clicks1 := 0
	// r.Post("/clicked1", nodeHandlerFunc(func() g.Node {
	// 	clicks1++
	// 	return h.P(
	// 		g.Text(fmt.Sprintf("Clicks: %d", clicks1)),
	// 	)
	// }))

	// t, err := template.New("").Parse(`
	// 	<p>{{.}}</p>
	// `)
	// if err != nil {
	// 	panic(err)
	// }
	// clicks2 := 0
	// r.Post("/clicked2", templateHandlerFunc(
	// 	t,
	// 	"",
	// 	func() any {
	// 		clicks2++
	// 		return clicks2
	// 	},
	// ))

	m := melody.New()

	m.HandleConnect(func(s *melody.Session) {
		slog.Debug(
			"ws connected",
			"remote addr", s.RemoteAddr().String(),
		)
		writeMessageToWebsocket(s, "TODO JEFF hello from server")
	})

	m.HandleMessage(func(s *melody.Session, b []byte) {
		message, err := unmarshalTaggedUnionJson(
			"type",
			map[string]interface{}{
				"login": &WebsocketLogin{},
				"send":  &WebsocketMessage{},
			},
			b,
		)
		if err != nil {
			slog.Error("json unmarshal error", "err", err)
			// TODO respond with an error message
			return
		}
		switch message.(type) {
		case *WebsocketLogin:
			slog.Debug("TODO JEFF got login", "msg", message)
		case *WebsocketMessage:
			slog.Debug("TODO JEFF got message", "msg", message)
		}

		// TODO finish the websocket handling

		// var message WebsocketMessage
		// if err := json.Unmarshal(b, &message); err != nil {
		// 	slog.Error("json unmarshal error", "err", err)
		// 	return
		// }
		// slog.Debug(
		// 	"received message",
		// 	"remote addr", s.RemoteAddr().String(),
		// 	"msg", message.Message,
		// )
		// writeMessageToWebsocket(s, fmt.Sprintf("TODO JEFF responding to \"%v\"", message.Message))
		// if err := writeNodeToWebSoccket(s, websocketForm()); err != nil {
		// 	slog.Error(
		// 		"error sending to websocket",
		// 		"remote addr", s.RemoteAddr().String(),
		// 		"err", err,
		// 	)
		// }
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

func websocketFormLogin(id string) g.Node {
	return h.FormEl(
		h.ID("ws-form"),
		g.Attr("hx-ws", "send"),
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
		g.Attr("hx-ws", "send"),
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

// TODO no?
func templateHandlerFunc(
	template *template.Template,
	name string,
	data func() any,
) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if err := template.ExecuteTemplate(w, name, data()); err != nil {
			slog.Error("error rendering template", "err", err)
		}
	}
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

func writeMessageToWebsocket(s *melody.Session, message string) {
	n := h.Div(
		h.ID("ws-messages"),
		g.Attr("hx-swap-oob", "beforeend"),
		h.Div(g.Text(message)),
	)
	if err := writeNodeToWebSoccket(s, n); err != nil {
		slog.Error(
			"error sending to websocket",
			"remote addr", s.RemoteAddr().String(),
			"err", err,
		)
	}
}

func writeNodeToWebSoccket(s *melody.Session, n g.Node) error {
	var w strings.Builder
	if err := n.Render(&w); err != nil {
		return err
	}
	return s.Write([]byte(w.String()))
}

func unmarshalTaggedUnionJson(
	tagName string,
	tags map[string]interface{},
	data []byte,
) (interface{}, error) {
	var tagOnly map[string]interface{}
	if err := json.Unmarshal(data, &tagOnly); err != nil {
		return nil, err
	}
	tag, ok := tagOnly[tagName]
	if !ok {
		return nil, fmt.Errorf("json missing tag: %v", tagName)
	}
	tagStr, ok := tag.(string)
	if !ok {
		return nil, fmt.Errorf("json tag wrong type: %v", reflect.TypeOf(tag))
	}
	v, ok := tags[tagStr]
	if !ok {
		return nil, fmt.Errorf("json had tag %v=%v, but no such type specified", tagName, tagStr)
	}
	if err := json.Unmarshal(data, v); err != nil {
		return nil, fmt.Errorf("error unmarshalling tag %v=%v into %v: %w", tagName, tagStr, reflect.TypeOf(v), err)
	}
	return v, nil
}
