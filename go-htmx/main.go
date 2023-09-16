package main

import (
	"experiments/livereload"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"sync"

	"github.com/go-chi/chi"
	g "github.com/maragudk/gomponents"
	c "github.com/maragudk/gomponents/components"
	. "github.com/maragudk/gomponents/html"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level: slog.LevelDebug,
		},
	)))

	r := chi.NewRouter()

	r.HandleFunc("/", createHandler(createPage([]g.Node{
		H1(g.Text("Hello, World!")),
		Button(
			g.Text("Click Me"),
			g.Attr("hx-post", "/clicked"),
			g.Attr("hx-target", "#target"),
		),
		Div(ID("target")),
	})))

	clicks := 0
	r.HandleFunc("/clicked", createHandlerF(func() g.Node {
		clicks++
		return g.Text(fmt.Sprintf("clicks: %d", clicks))
	}))

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

func createPage(body []g.Node) g.Node {
	return c.HTML5(c.HTML5Props{
		Title:    "Experiment",
		Language: "en",
		Head: []g.Node{
			Script(Src("https://unpkg.com/htmx.org@1.9.5")),
			livereload.NewScript(),
		},
		Body: body,
	})
}

func createHandler(node g.Node) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		node.Render(w)
	}
}

func createHandlerF(f func() g.Node) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		f().Render(w)
	}
}
