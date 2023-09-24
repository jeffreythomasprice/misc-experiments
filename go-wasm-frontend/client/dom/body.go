package dom

import "syscall/js"

type Body interface {
	Element
}

type bodyImpl struct {
	elementImpl
}

var _ Body = (*bodyImpl)(nil)

func newBody(value js.Value) *bodyImpl {
	return &bodyImpl{newElement(value)}
}
