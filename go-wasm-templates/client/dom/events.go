package dom

import "syscall/js"

type Event struct {
	this js.Value
	args []js.Value
}

type EventHandler func(e Event)

func (e Event) PreventDefault() {
	e.args[0].Call("preventDefault")
}

func (handler EventHandler) JsFunc() js.Func {
	return js.FuncOf(func(this js.Value, args []js.Value) any {
		handler(Event{this, args})
		return nil
	})
}
