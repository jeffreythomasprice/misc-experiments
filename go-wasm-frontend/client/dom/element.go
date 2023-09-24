package dom

import "syscall/js"

type Element interface {
	Node
	Children() []Node
	InnerText() string
	SetInnerText(s string)
	InnerHTML() string
	SetInnerHTML(s string)
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

func (e elementImpl) Children() []Node {
	children := e.Get("children")
	len := children.Length()
	results := make([]Node, 0, len)
	for i := 0; i < len; i++ {
		results = append(results, newNode(children.Call("item", i)))
	}
	return results
}

func (e elementImpl) InnerText() string {
	return e.Get("innerText").String()
}

func (e elementImpl) SetInnerText(s string) {
	e.Set("innerText", s)
}

func (e elementImpl) InnerHTML() string {
	return e.Get("innerHTML").String()
}

func (e elementImpl) SetInnerHTML(s string) {
	e.Set("innerHTML", s)
}

// AddEventListener implements EventTarget.
func (e elementImpl) AddEventListener(typ string, listener func(args []js.Value)) {
	e.Call("addEventListener", typ, js.FuncOf(func(this js.Value, args []js.Value) any {
		listener(args)
		return nil
	}))
}
