package main

import (
	"client/swap"
	"embed"
	"html/template"
	"io"
	"log/slog"
	"shared"
	"syscall/js"
)

//go:embed assets/templates
var templateAssets embed.FS

func main() {
	shared.InitSlog()

	templates, err := template.ParseFS(templateAssets, "assets/templates/**")
	if err != nil {
		fail("failed to parse templates", err)
	}

	clicks := 0

	if err := swap.Swap(
		"body",
		swap.InnerHTML,
		func(w io.Writer) error {
			return templates.ExecuteTemplate(w, "test", map[string]any{
				"msg": "Hello, World!",
			})
		},
		map[string]swap.EventHandler{
			"click": func(this js.Value, args []js.Value) {
				clicks++

				if err := swap.Swap(
					"#clicks",
					swap.InnerHTML,
					func(w io.Writer) error {
						return templates.ExecuteTemplate(w, "click", map[string]any{
							"count": clicks,
						})
					},
					nil,
				); err != nil {
					fail("failed to swap in click content", err)
				}
			},
		},
	); err != nil {
		fail("failed to swap in content", err)
	}

	select {}
}

func fail(msg string, err error) {
	slog.Error(msg, "err", err)
	panic("fatal error")
}
