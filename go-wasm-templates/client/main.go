package main

import (
	"client/dom"
	"client/swap"
	"io"
	"log/slog"
	"shared"
	"syscall/js"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/html"
)

func main() {
	shared.InitSlog()

	loginForm, err := loginForm()
	if err != nil {
		panic(err)
	}
	if err := swap.Swap(
		"body",
		swap.InnerHTML,
		loginForm.Render,
		map[string]swap.EventHandler{
			"submit": func(this js.Value, args []js.Value) {
				e := args[0]
				e.Call("preventDefault")

				username := dom.MustQuerySelector("input[name='username']").Get("value").String()
				password := dom.MustQuerySelector("input[name='password']").Get("value").String()

				slog.Debug("TODO JEFF submit", "username", username, "password", password)
			},
		},
	); err != nil {
		panic(err)
	}

	select {}
}

type Nodes []Node

func (nodes Nodes) Render(w io.Writer) error {
	for _, node := range nodes {
		if err := node.Render(w); err != nil {
			return err
		}
	}
	return nil
}

func loginForm() (Nodes, error) {
	return Nodes{
		Div(
			Label(
				For("username"),
				Text("Username:"),
			),
			Input(
				FormAttr("loginForm"),
				Name("username"),
				Placeholder("Username"),
				Type("text"),
			),
		),
		Div(
			Label(
				For("password"),
				Text("Password:"),
			),
			Input(
				FormAttr("loginForm"),
				Name("password"),
				Placeholder("Password"),
				Type("password"),
			),
		),
		FormEl(
			ID("loginForm"),
			Attr("go-click", "submit"),
			Button(
				Type("submit"),
				Text("Log In"),
			),
		),
	}, nil
}
