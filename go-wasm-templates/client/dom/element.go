package dom

import (
	"errors"
	"syscall/js"
)

type Element struct {
	js.Value
	errors []error
}

var ErrSelectorNotFound = errors.New("no such element for selectors")

var _ Renderer = (*Element)(nil)

func NewElement(tagName string, renderers ...Renderer) *Element {
	result := &Element{
		Value:  js.Global().Get("document").Call("createElement", tagName),
		errors: nil,
	}
	for _, r := range renderers {
		if err := r.apply(result); err != nil {
			result.errors = append(result.errors, err)
		}
	}
	return result
}

func (e *Element) Error() error {
	return errors.Join(e.errors...)
}

// apply implements Renderer.
func (e *Element) apply(target *Element) error {
	return Append(target.Value, []js.Value{e.Value})
}
