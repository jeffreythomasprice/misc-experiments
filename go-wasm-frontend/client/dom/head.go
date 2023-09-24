package dom

import "syscall/js"

type Head interface {
	Element
}

type headImpl struct {
	elementImpl
}

var _ Head = (*headImpl)(nil)

func newHead(value js.Value) *headImpl {
	return &headImpl{newElement(value)}
}
