package dom

import (
	"errors"
	"fmt"
	"syscall/js"
)

type Selector func() (js.Value, error)

var ErrSelectorNotFound = errors.New("no such element for selector")

func QuerySelector(selectors string) Selector {
	return func() (js.Value, error) {
		result := js.Global().Get("document").Call("querySelector", selectors)
		if !result.Truthy() {
			return js.Null(), fmt.Errorf("%w, was looking for %v", ErrSelectorNotFound, selectors)
		}
		return result, nil
	}
}
