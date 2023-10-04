package main

import (
	"client/dom"
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

	if err := swap.Swap(
		"body",
		swap.InnerHTML,
		templ(templates, "login", nil),
		map[string]swap.EventHandler{
			"submit": func(this js.Value, args []js.Value) {
				e := args[0]
				e.Call("preventDefault")

				username := dom.MustQuerySelector("input[name='username']").Get("value")
				password := dom.MustQuerySelector("input[name='password']").Get("value")

				slog.Debug("TODO JEFF submit", "username", username, "password", password)
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

func templ(t *template.Template, name string, data any) swap.Generator {
	return func(w io.Writer) error {
		return t.ExecuteTemplate(w, name, data)
	}
}
