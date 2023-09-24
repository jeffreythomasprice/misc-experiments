package dom

import (
	"syscall/js"
)

type Node interface {
	jsValue() js.Value
	AppendChild(other Node)
	RemoveChild(other Node)
}

type nodeImpl struct {
	js.Value
}

var _ Node = nodeImpl{}

func newNode(value js.Value) nodeImpl {
	return nodeImpl{value}
}

func (n nodeImpl) jsValue() js.Value {
	return n.Value
}

func (n nodeImpl) AppendChild(other Node) {
	n.Call("appendChild", other.jsValue())
}

func (n nodeImpl) RemoveChild(other Node) {
	n.Call("removeChild", other.jsValue())
}
