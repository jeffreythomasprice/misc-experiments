package dom

import "syscall/js"

type Document struct {
	*Node
}

func NewDocument() *Document {
	return &Document{NewNode(js.Global().Get("document"))}
}

func (d *Document) Body() *Body {
	return &Body{NewElement(d.Get("body"))}
}

func (d *Document) CreateElement(tagName string) *Element {
	return NewElement(d.Call("createElement", tagName))
}

func (d *Document) QuerySelector(selectors string) *Element {
	result := d.Call("querySelector", selectors)
	if result.Truthy() {
		return NewElement(result)
	}
	return nil
}
