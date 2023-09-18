package dom

import "syscall/js"

type Element struct {
	*Node
}

func NewElement(value js.Value) *Element {
	return &Element{NewNode(value)}
}

func (e *Element) Children() []*Node {
	children := e.Get("children")
	len := children.Length()
	results := make([]*Node, 0, len)
	for i := 0; i < len; i++ {
		results = append(results, NewNode(children.Call("item", i)))
	}
	return results
}

func (e *Element) InnerText() string {
	return e.Get("innerText").String()
}

func (e *Element) SetInnerText(s string) {
	e.Set("innerText", s)
}

func (e *Element) InnerHTML() string {
	return e.Get("innerHTML").String()
}

func (e *Element) SetInnerHTML(s string) {
	e.Set("innerHTML", s)
}
