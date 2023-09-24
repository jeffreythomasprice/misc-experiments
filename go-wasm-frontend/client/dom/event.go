package dom

import "syscall/js"

type Event interface {
	Target() js.Value
	PreventDefault()
}

type eventImpl struct {
	js.Value
}

var _ Event = eventImpl{}

func newEvent(value js.Value) eventImpl {
	return eventImpl{value}
}

func (e eventImpl) Target() js.Value {
	return e.Get("target")
}

func (e eventImpl) PreventDefault() {
	e.Call("preventDefault")
}
