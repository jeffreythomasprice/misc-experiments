package main

import (
	"client/websockets"
	"context"
	"errors"
	"log/slog"
	"shared"
	"strings"
	"syscall/js"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/html"
)

var errSelectorsNotFound = errors.New("no element found for selectors")

func main() {
	shared.InitSlog()

	node := H1(Text("Hello, World!"))
	if err := appendNode("body", node); err != nil {
		panic(err)
	}

	go liveReload("ws://127.0.0.1:8000/_liveReload")

	go func() {
		outgoing, incoming := websockets.NewBuilder("ws://127.0.0.1:8000/ws").
			// TODO backoff strategy
			Reconnect(websockets.Every(1)).
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

	select {}
}

func appendNode(selectors string, node Node) error {
	target, err := querySelector(selectors)
	if err != nil {
		return err
	}

	elements, err := createDomElements(node)
	if err != nil {
		return err
	}

	if len(elements) == 0 {
		target.Call("remove")
		return nil
	}
	target.Call("replaceWith", elements[0])
	last := elements[0]
	for _, e := range elements[1:] {
		last.Call("after", e)
		last = e
	}
	return nil
}

func createDomElements(node Node) ([]js.Value, error) {
	var s strings.Builder
	if err := node.Render(&s); err != nil {
		return nil, err
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
