package dom

import "syscall/js"

type Event struct {
	js.Value
}

func NewEvent(value js.Value) *Event {
	return &Event{value}
}

func (e *Event) Target() js.Value {
	return e.Get("target")
}

func (e *Event) PreventDefault() {
	e.Call("preventDefault")
}
