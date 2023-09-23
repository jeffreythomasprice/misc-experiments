package main

import (
	"encoding/json"
	"experiments/livereload"
	"fmt"
	"html/template"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"sync"

	"github.com/go-chi/chi"
	g "github.com/maragudk/gomponents"
	c "github.com/maragudk/gomponents/components"
	h "github.com/maragudk/gomponents/html"
	"github.com/olahol/melody"
)

type WebsocketMessage struct {
	Message string             `json:"message"`
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

	r.Get("/", pageHandlerFunc(
		func() []g.Node {
			return []g.Node{liveReloadScript}
		},
		func() []g.Node {
			return []g.Node{
				h.H1(g.Text("Hello, World!")),

				h.Button(
					g.Text("Click Me"),
					g.Attr("hx-post", "/clicked1"),
					g.Attr("hx-target", "#target1"),
				),
				h.Div(h.ID("target1")),

				h.Button(
					g.Text("Also Click Me"),
					g.Attr("hx-post", "/clicked2"),
					g.Attr("hx-target", "#target2"),
				),
				h.Div(h.ID("target2")),

				h.Div(
					g.Attr("hx-ws", "connect:/ws"),
					h.FormEl(
						h.ID("ws-form"),
						g.Attr("hx-ws", "send"),
						h.Input(
							h.Name("message"),
							h.AutoFocus(),
						),
					),
					h.Div(
						h.ID("ws-messages"),
					),
				),
			}
		},
	))

	clicks1 := 0
	r.Post("/clicked1", nodeHandlerFunc(func() g.Node {
		clicks1++
		return h.P(
			g.Text(fmt.Sprintf("Clicks: %d", clicks1)),
		)
	}))

	t, err := template.New("").Parse(`
		<p>{{.}}</p>
	`)
	if err != nil {
		panic(err)
	}
	clicks2 := 0
	r.Post("/clicked2", templateHandlerFunc(
		t,
		"",
		func() any {
			clicks2++
			return clicks2
		},
	))

	m := melody.New()
	m.HandleConnect(func(s *melody.Session) {
		slog.Debug(
			"ws connected",
			"remote addr", s.RemoteAddr().String(),
		)
		if err := writeToWebSocket(s, func() g.Node {
			return h.Div(
				h.ID("ws-messages"),
				g.Attr("hx-swap-oob", "beforeend"),
				h.Div(g.Text("TODO JEFF hello from server")),
			)
		}); err != nil {
			slog.Error(
				"error sending to websocket",
				"remote addr", s.RemoteAddr().String(),
				"err", err,
			)
		}
	})
	m.HandleMessage(func(s *melody.Session, b []byte) {
		var message WebsocketMessage
		if err := json.Unmarshal(b, &message); err != nil {
			slog.Error("json unmarshal error", "err", err)
			return
		}
		slog.Debug(
			"received message",
			"remote addr", s.RemoteAddr().String(),
			"msg", message.Message,
		)
		// TODO deduplicate the writing of messages
		if err := writeToWebSocket(s, func() g.Node {
			return h.Div(
				h.ID("ws-messages"),
				g.Attr("hx-swap-oob", "beforeend"),
				h.Div(g.Textf("TODO JEFF responding to \"%v\"", message.Message)),
			)
		}); err != nil {
			slog.Error(
				"error sending to websocket",
				"remote addr", s.RemoteAddr().String(),
				"err", err,
			)
		}
		if err := writeToWebSocket(s, func() g.Node {
			// TODO deduplicate the form
			return h.FormEl(
				h.ID("ws-form"),
				g.Attr("hx-ws", "send"),
				h.Input(
					h.Name("message"),
					h.AutoFocus(),
				),
			)
		}); err != nil {
			slog.Error(
				"error sending to websocket",
				"remote addr", s.RemoteAddr().String(),
				"err", err,
			)
		}
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

func pageHandlerFunc(head, body func() []g.Node) http.HandlerFunc {
	return nodeHandlerFunc(func() g.Node {
		return c.HTML5(c.HTML5Props{
			Title:    "Experiment",
			Language: "en",
			Head: append(
				[]g.Node{
					h.Script(h.Src("https://unpkg.com/htmx.org@1.9.5")),
					h.Script(h.Src("https://unpkg.com/htmx.org@1.9.5/dist/ext/ws.js")),
				},
				head()...,
			),
			Body: body(),
		})
	})
}

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

func nodeHandlerFunc(f func() g.Node) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if err := f().Render(w); err != nil {
			slog.Error("error rendering node", "err", err)
		}
	}
}

func writeToWebSocket(s *melody.Session, f func() g.Node) error {
	var w strings.Builder
	if err := f().Render(&w); err != nil {
		return err
	}
	return s.Write([]byte(w.String()))
}
