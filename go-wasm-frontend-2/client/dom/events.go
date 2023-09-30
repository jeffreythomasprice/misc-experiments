package dom

import "syscall/js"

type Event struct {
	js.Value
}

func (e Event) PreventDefault() {
	e.Call("preventDefault")
}

func AddEventListener(selector Selector, eventName string, f func(args []js.Value)) error {
	target, err := selector()
	if err != nil {
		return err
	}

	target.Call("addEventListener", eventName, js.FuncOf(func(this js.Value, args []js.Value) any {
		f(args)
		return nil
	}))

	return nil
}

func OnClick(selector Selector, f func(e Event)) error {
	return AddEventListener(selector, "click", func(args []js.Value) {
		f(Event{args[0]})
	})
}
