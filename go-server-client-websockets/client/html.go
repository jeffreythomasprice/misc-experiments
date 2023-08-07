package main

import (
	"errors"
	"fmt"
	"syscall/js"
)

/*
DomElement represents some HTML element tree that can be added as a child to another element.
*/
type DomElement struct {
	value js.Value
}

var ErrMismatchedParents = errors.New("argument is not a child node of this node")
var ErrAlreadyHasParent = errors.New("argument is already a child node of some other parent node")

func NewDomElementFromHtmlString(html string) (results []*DomElement, err error) {
	// TODO JEFF verify if exceptions are possible here with malformed html
	defer func() {
		if r := recover(); r != nil {
			err = errors.New(fmt.Sprintf("%v", r))
		}
	}()
	tempParent := &DomElement{js.Global().Get("document").Call("createElement", "div")}
	tempParent.value.Set("innerHTML", html)
	results = tempParent.Children()
	err = tempParent.RemoveAllChildren()
	return
}

func GetDomElementByQuerySelector(selectors string) (result *DomElement, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = errors.New(fmt.Sprintf("%v", r))
		}
	}()
	value := js.Global().Get("document").Call("querySelector", selectors)
	result = &DomElement{value}
	return
}

func GetAllDomElementsByQuerySelector(selectors string) (results []*DomElement, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = errors.New(fmt.Sprintf("%v", r))
		}
	}()
	values := js.Global().Get("document").Call("querySelectorAll", selectors)
	results = make([]*DomElement, 0, values.Length())
	for i := 0; i < values.Length(); i++ {
		results = append(results, &DomElement{values.Index(i)})
	}
	return
}

func (elem *DomElement) Equals(other *DomElement) bool {
	return elem.value.Equal(other.value)
}

func (elem *DomElement) Children() []*DomElement {
	value := elem.value.Get("children")
	if !value.Truthy() {
		return nil
	}
	results := make([]*DomElement, 0, value.Length())
	for i := 0; i < value.Length(); i++ {
		results = append(results, &DomElement{value.Index(i)})
	}
	return results
}

func (elem *DomElement) Parent() *DomElement {
	result := elem.value.Get("parentElement")
	if !result.Truthy() {
		return nil
	}
	return &DomElement{result}
}

func (elem *DomElement) RemoveChild(child *DomElement) error {
	if !child.Parent().Equals(elem) {
		return ErrMismatchedParents
	}
	elem.value.Call("removeChild", child.value)
	return nil
}

func (elem *DomElement) RemoveAllChildren() error {
	children := elem.Children()
	for _, child := range children {
		if err := elem.RemoveChild(child); err != nil {
			return err
		}
	}
	return nil
}

func (elem *DomElement) AppendChild(child *DomElement) error {
	if child.Parent() != nil {
		return ErrAlreadyHasParent
	}
	elem.value.Call("appendChild", child.value)
	return nil
}
