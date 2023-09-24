package dom

import "syscall/js"

type Element interface {
	Node
	ID() string
	SetID(s string)
	Children() []Node
	InnerText() string
	SetInnerText(s string)
	InnerHTML() string
	SetInnerHTML(s string)
	GetAttribute(name string) string
	SetAttribute(name, value string)
	RemoveAttribute(name string) string
}

type elementImpl struct {
	nodeImpl
}

var _ Element = elementImpl{}
var _ EventTarget = elementImpl{}

func newElement(value js.Value) elementImpl {
	return elementImpl{newNode(value)}
}

func AsElement(n Node) Element {
	return newElement(n.jsValue())
}

func (e elementImpl) ID() string {
	return e.jsValue().Get("id").String()
}

func (e elementImpl) SetID(s string) {
	e.jsValue().Set("id", s)
}

func (e elementImpl) Children() []Node {
	children := e.jsValue().Get("children")
	len := children.Length()
	results := make([]Node, 0, len)
	for i := 0; i < len; i++ {
		results = append(results, newNode(children.Call("item", i)))
	}
	return results
}

func (e elementImpl) InnerText() string {
	return e.jsValue().Get("innerText").String()
}

func (e elementImpl) SetInnerText(s string) {
	e.jsValue().Set("innerText", s)
}

func (e elementImpl) InnerHTML() string {
	return e.jsValue().Get("innerHTML").String()
}

func (e elementImpl) SetInnerHTML(s string) {
	e.jsValue().Set("innerHTML", s)
}

func (e elementImpl) GetAttribute(name string) string {
	return e.jsValue().Call("getAttribute", name).String()
}

func (e elementImpl) SetAttribute(name string, value string) {
	e.jsValue().Call("setAttribute", name, value)
}

func (e elementImpl) RemoveAttribute(name string) string {
	return e.jsValue().Call("removeAttribute", name).String()
}

func (e elementImpl) AddEventListener(typ string, listener func(args []js.Value)) {
	e.jsValue().Call("addEventListener", typ, js.FuncOf(func(this js.Value, args []js.Value) any {
		listener(args)
		return nil
	}))
}
