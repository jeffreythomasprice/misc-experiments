package dom

import "syscall/js"

type HTMLElement interface {
	Element
	Focus()
}

type htmlElementImpl struct {
	elementImpl
}

var _ HTMLElement = htmlElementImpl{}

func newHTMLElement(value js.Value) htmlElementImpl {
	return htmlElementImpl{newElement(value)}
}

func AsHTMLElement(n Node) HTMLElement {
	return newHTMLElement(n.jsValue())
}

func (e htmlElementImpl) Focus() {
	e.jsValue().Call("focus")
}
