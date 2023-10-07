package dom

import (
	"syscall/js"
)

type Event struct {
	this js.Value
	args []js.Value
}

type EventHandlerFunc func(e Event)

type eventHandler struct {
	name    string
	handler EventHandlerFunc
}

var _ Renderer = (*eventHandler)(nil)

func EventHandler(name string, f EventHandlerFunc) Renderer {
	return &eventHandler{name, f}
}

// apply implements Renderer.
func (e *eventHandler) apply(target *Element) error {
	target.Call(
		"addEventListener",
		e.name,
		js.FuncOf(func(this js.Value, args []js.Value) any {
			e.handler(Event{this, args})
			return nil
		}),
	)
	return nil
}

func (e Event) This() *Element {
	return &Element{
		Value:  e.this,
		errors: nil,
	}
}

func (e Event) PreventDefault() {
	e.args[0].Call("preventDefault")
}
