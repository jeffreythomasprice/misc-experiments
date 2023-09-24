package dom

import (
	"syscall/js"
)

type Document interface {
	Node
	Body() Body
	CreateElement(tagName string) Element
	QuerySelector(selectors string) Element
	QuerySelectorAll(selectors string) []Element
}

type documentImpl struct {
	nodeImpl
}

var _ Document = documentImpl{}

func GetDocument() documentImpl {
	return documentImpl{newNode(js.Global().Get("document"))}
}

func (d documentImpl) Body() Body {
	return newBody(d.jsValue().Get("body"))
}

func (d documentImpl) Head() Head {
	return newHead(d.jsValue().Get("head"))
}

func (d documentImpl) CreateElement(tagName string) Element {
	return newElement(d.jsValue().Call("createElement", tagName))
}

func (d documentImpl) QuerySelector(selectors string) Element {
	result := d.jsValue().Call("querySelector", selectors)
	if result.Truthy() {
		return newElement(result)
	}
	return nil
}

func (d documentImpl) QuerySelectorAll(selectors string) []Element {
	jsResults := d.jsValue().Call("querySelectorAll", selectors)
	results := make([]Element, jsResults.Length())
	for i := 0; i < jsResults.Length(); i++ {
		results[i] = newElement(jsResults.Index(i))
	}
	return results
}
