package dom

import "syscall/js"

type HTMLFormElement interface {
	HTMLElement
	OnSubmit(f func(e SubmitEvent))
}

type htmlFormElementImpl struct {
	htmlElementImpl
}

var _ HTMLFormElement = htmlFormElementImpl{}

func newHTMLFormElement(value js.Value) htmlFormElementImpl {
	return htmlFormElementImpl{newHTMLElement(value)}
}

func AsHTMLFormElement(n Node) HTMLFormElement {
	return newHTMLFormElement(n.jsValue())
}

func (e htmlFormElementImpl) OnSubmit(f func(e SubmitEvent)) {
	e.AddEventListener("submit", func(args []js.Value) {
		f(newSubmitEvent(args[0]))
	})
}
