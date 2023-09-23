package dom

import "syscall/js"

type HTMLElement struct {
	*Element
}

func NewHTMLElement(value js.Value) *HTMLElement {
	return &HTMLElement{NewElement(value)}
}
