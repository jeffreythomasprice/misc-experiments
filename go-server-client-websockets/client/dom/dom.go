package dom

import (
	"errors"
	"syscall/js"
)

/*
DomElement represents some HTML element tree that can be added as a child to another element.
*/
type DomElement struct {
	js.Value
}

var ErrMismatchedParents = errors.New("argument is not a child node of this node")
var ErrAlreadyHasParent = errors.New("argument is already a child node of some other parent node")

func NewDomElementFromHtmlString(html string) (results []*DomElement, err error) {
	defer commonErrorHandling(&err)
	tempParent := &DomElement{js.Global().Get("document").Call("createElement", "div")}
	tempParent.Value.Set("innerHTML", html)
	results = tempParent.Children()
	err = tempParent.RemoveAllChildren()
	return
}

func MustNewDomElementFromHtmlString(html string) []*DomElement {
	results, err := NewDomElementFromHtmlString(html)
	if err != nil {
		panic(err)
	}
	return results
}

func GetDomElementById(id string) (result *DomElement, err error) {
	defer commonErrorHandling(&err)
	value := js.Global().Get("document").Call("getElementById", id)
	if !value.Truthy() {
		return nil, nil
	}
	result = &DomElement{value}
	return
}

func MustGetDomElementById(id string) *DomElement {
	result, err := GetDomElementById(id)
	if err != nil {
		panic(err)
	}
	return result
}

func GetDomElementByQuerySelector(selectors string) (result *DomElement, err error) {
	defer commonErrorHandling(&err)
	value := js.Global().Get("document").Call("querySelector", selectors)
	if !value.Truthy() {
		return nil, nil
	}
	result = &DomElement{value}
	return
}

func MustGetDomElementByQuerySelector(selectors string) *DomElement {
	result, err := GetDomElementByQuerySelector(selectors)
	if err != nil {
		panic(err)
	}
	return result
}

func GetAllDomElementsByQuerySelector(selectors string) (results []*DomElement, err error) {
	defer commonErrorHandling(&err)
	values := js.Global().Get("document").Call("querySelectorAll", selectors)
	if values.Length() == 0 {
		return nil, nil
	}
	results = make([]*DomElement, 0, values.Length())
	for i := 0; i < values.Length(); i++ {
		results = append(results, &DomElement{values.Index(i)})
	}
	return
}

func MustGetAllDomElementsByQuerySelector(selectors string) []*DomElement {
	results, err := GetAllDomElementsByQuerySelector(selectors)
	if err != nil {
		panic(err)
	}
	return results
}

func (elem *DomElement) Equals(other *DomElement) bool {
	return elem.Value.Equal(other.Value)
}

func (elem *DomElement) Children() []*DomElement {
	value := elem.Value.Get("children")
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
	result := elem.Value.Get("parentElement")
	if !result.Truthy() {
		return nil
	}
	return &DomElement{result}
}

func (elem *DomElement) RemoveChild(child *DomElement) error {
	if !child.Parent().Equals(elem) {
		return ErrMismatchedParents
	}
	elem.Value.Call("removeChild", child.Value)
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
	elem.Value.Call("appendChild", child.Value)
	return nil
}

func (elem *DomElement) ReplaceChildren(newChildren []*DomElement) error {
	if err := elem.RemoveAllChildren(); err != nil {
		return err
	}
	for _, child := range newChildren {
		if err := elem.AppendChild(child); err != nil {
			return err
		}
	}
	return nil
}
