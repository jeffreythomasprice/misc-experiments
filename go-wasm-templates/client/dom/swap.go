package dom

import (
	"fmt"
	"syscall/js"
)

type SwapFunc func(target js.Value, children []js.Value) error

func (e *Element) Swap(selectors string, f SwapFunc) error {
	if err := e.Error(); err != nil {
		return err
	}

	target := js.Global().Get("document").Call("querySelector", selectors)
	if !target.Truthy() {
		return fmt.Errorf("%w: %v", ErrSelectorNotFound, selectors)
	}

	return f(target, []js.Value{e.Value})
}

func Append(target js.Value, children []js.Value) error {
	target.Call("append", valueSliceAsAnySlice(children)...)
	return nil
}

func ReplaceChildren(target js.Value, children []js.Value) error {
	target.Call("replaceChildren", valueSliceAsAnySlice(children)...)
	return nil
}

func valueSliceAsAnySlice(input []js.Value) []any {
	result := make([]any, 0, len(input))
	for _, x := range input {
		result = append(result, x)
	}
	return result
}
