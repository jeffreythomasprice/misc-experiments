package dom

import "syscall/js"

type HTMLInputElement interface {
	HTMLElement
	Value() string
	SetValue(s string)
}

type htmlInputElementImpl struct {
	htmlElementImpl
}

var _ HTMLInputElement = htmlInputElementImpl{}

func newHTMLInputElement(value js.Value) htmlInputElementImpl {
	return htmlInputElementImpl{newHTMLElement(value)}
}

func AsHTMLInputElement(n Node) HTMLInputElement {
	return newHTMLInputElement(n.jsValue())
}

func (e htmlInputElementImpl) Value() string {
	return e.Get("value").String()
}

func (e htmlInputElementImpl) SetValue(s string) {
	e.Set("value", s)
}
