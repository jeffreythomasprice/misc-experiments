package main

import (
	"client/websockets"
	"context"
	"errors"
	"log/slog"
	"shared"
	"strings"
	"syscall/js"
	"time"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/html"
)

var errSelectorsNotFound = errors.New("no element found for selectors")

func main() {
	shared.InitSlog()

	go liveReload("ws://127.0.0.1:8000/_liveReload")

	go func() {
		outgoing, incoming := websockets.NewBuilder("ws://127.0.0.1:8000/ws").
			Reconnect(websockets.Backoff(time.Second*1, time.Second*5)).
			Build(context.Background())
		go func() {
			for msg := range incoming {
				if msg.IsTextMessage() {
					slog.Debug("TODO received text message", "msg", msg.Text())
				} else if msg.IsBinaryMessage() {
					slog.Debug("TODO received binary message", "msg", string(msg.Binary()))
				}
			}
		}()
		outgoing <- websockets.NewTextMessage("TODO test text message")
		outgoing <- websockets.NewBinaryMessage([]byte("TODO test binary message"))
	}()

	root := "body"

	signInPage(
		root,
		func(username, password string) {
			slog.Debug("TODO attempt to sign in", "username", username, "password", password)
		},
	)

	select {}
}

func signInPage(selectors string, callback func(username, password string)) {
	if err := replaceChildrenWithNodes(
		selectors,
		Div(
			ID("login"),
			Label(
				For("username"),
				Text("Username"),
			),
			Input(
				Name("username"),
				FormAttr("loginForm"),
				Type("text"),
				Placeholder("Username"),
			),
			Label(
				For("password"),
				Text("Password"),
			),
			Input(
				Name("password"),
				FormAttr("loginForm"),
				Type("password"),
				Placeholder("Password"),
			),
			FormEl(
				ID("loginForm"),
				Button(
					Text("Sign In"),
					Type("submit"),
				),
			),
			Div(ID("errorMessages")),
		),
	); err != nil {
		renderError("login", err)
		return
	}

	// TODO JEFF wrapper for basic events
	js.
		Global().
		Get("document").
		Call("getElementById", "loginForm").
		Call(
			"addEventListener",
			"submit",
			js.FuncOf(func(this js.Value, args []js.Value) any {
				e := args[0]
				e.Call("preventDefault")

				formElements := e.Get("target").Get("elements")

				usernameInput := formElements.Get("username")
				username := usernameInput.Get("value").String()

				passwordInput := formElements.Get("password")
				password := passwordInput.Get("value").String()

				if len(username) == 0 {
					errorMessage("#login > #errorMessages", "Must provide username")
					return nil
				}

				if len(password) == 0 {
					errorMessage("#login > #errorMessages", "Must provide password")
					return nil
				}

				callback(username, password)

				return nil
			}),
		)
}

func renderError(page string, err error) {
	slog.Error("error rendering", "page", page, "err", err)
	errorMessage("body", "Render error")
}

func errorMessage(selectors, msg string) {
	if err := replaceChildrenWithNodes(
		selectors,
		Div(
			StyleAttr("color: red; font-weight: bold"),
			Text(msg),
		),
	); err != nil {
		slog.Error("error rendering dom nodes for another error message", "err", err)
	}
}

func appendNodes(selectors string, nodes ...Node) error {
	return domSwapper(selectors, nodes, func(target js.Value, elements []js.Value) error {
		for _, e := range elements {
			target.Call("append", e)
		}
		return nil
	})
}

func replaceChildrenWithNodes(selectors string, nodes ...Node) error {
	return domSwapper(selectors, nodes, func(target js.Value, elements []js.Value) error {
		currentChildren := target.Get("children")
		for currentChildren.Length() > 0 {
			currentChildren.Index(0).Call("remove")
		}
		for _, e := range elements {
			target.Call("append", e)
		}
		return nil
	})
}

func domSwapper(selectors string, nodes []Node, f func(target js.Value, elements []js.Value) error) error {
	target, err := querySelector(selectors)
	if err != nil {
		return err
	}

	elements, err := createDomElements(nodes...)
	if err != nil {
		return err
	}

	if err := f(target, elements); err != nil {
		return err
	}

	return nil
}

func createDomElements(nodes ...Node) ([]js.Value, error) {
	var s strings.Builder
	for _, n := range nodes {
		if err := n.Render(&s); err != nil {
			return nil, err
		}
	}
	temp := js.Global().Get("document").Call("createElement", "div")
	temp.Set("innerHTML", s.String())
	children := temp.Get("children")
	results := make([]js.Value, 0)
	for children.Length() > 0 {
		child := children.Index(0)
		temp.Call("removeChild", child)
		results = append(results, child)
	}
	return results, nil
}

func querySelector(selectors string) (js.Value, error) {
	value := js.Global().Get("document").Call("querySelector", selectors)
	if !value.Truthy() {
		return js.Null(), errSelectorsNotFound
	}
	return value, nil
}
