package dom

import (
	"syscall/js"
)

type Document interface {
	Node
	Body() Body
	CreateElement(tagName string) Element
	QuerySelector(selectors string) Element
}

type documentImpl struct {
	nodeImpl
}

var _ Document = (*documentImpl)(nil)

func GetDocument() *documentImpl {
	return &documentImpl{newNode(js.Global().Get("document"))}
}

func (d *documentImpl) Body() Body {
	return newBody(d.Get("body"))
}

func (d *documentImpl) CreateElement(tagName string) Element {
	return newElement(d.Call("createElement", tagName))
}

func (d *documentImpl) QuerySelector(selectors string) Element {
	result := d.Call("querySelector", selectors)
	if result.Truthy() {
		return newElement(result)
	}
	return nil
}
