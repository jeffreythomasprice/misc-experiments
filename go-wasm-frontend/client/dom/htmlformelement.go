package dom

import "syscall/js"

type HTMLFormElement struct {
	*HTMLElement
}

func NewHTMLFormElement(value js.Value) *HTMLFormElement {
	return &HTMLFormElement{NewHTMLElement(value)}
}

func (e *HTMLFormElement) OnSubmit(f func(e *SubmitEvent)) {
	e.AddEventListener("submit", func(args []js.Value) {
		f(NewSubmitEvent(args[0]))
	})
}
