package dom

import "syscall/js"

type HTMLInputElement struct {
	*HTMLElement
}

func NewHTMLInputElement(value js.Value) *HTMLInputElement {
	return &HTMLInputElement{NewHTMLElement(value)}
}

func (e *HTMLInputElement) Value() string {
	return e.Get("value").String()
}

func (e *HTMLInputElement) SetValue(s string) {
	e.Set("value", s)
}
