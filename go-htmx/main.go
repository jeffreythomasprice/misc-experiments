package main

import (
	"experiments/livereload"
	"fmt"
	"html/template"
	"log/slog"
	"net/http"
	"os"
	"sync"

	"github.com/go-chi/chi"
	g "github.com/maragudk/gomponents"
	c "github.com/maragudk/gomponents/components"
	h "github.com/maragudk/gomponents/html"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level: slog.LevelDebug,
		},
	)))

	r := chi.NewRouter()

	r.Get("/", pageHandlerFunc(func() []g.Node {
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
		}
	}))

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

	livereload.HandleFunc(r)

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

func pageHandlerFunc(body func() []g.Node) http.HandlerFunc {
	return nodeHandlerFunc(func() g.Node {
		return c.HTML5(c.HTML5Props{
			Title:    "Experiment",
			Language: "en",
			Head: []g.Node{
				h.Script(h.Src("https://unpkg.com/htmx.org@1.9.5")),
				livereload.NewScript(),
			},
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
