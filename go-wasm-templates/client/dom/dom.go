package dom

import (
	"fmt"
	"syscall/js"
)

func QuerySelector(selectors string) (js.Value, error) {
	result := js.Global().Get("document").Call("querySelector", selectors)
	if !result.Truthy() {
		return js.Null(), fmt.Errorf("no such element for selectors: %v", selectors)
	}
	return result, nil
}

func MustQuerySelector(selectors string) js.Value {
	result, err := QuerySelector(selectors)
	if err != nil {
		panic(err)
	}
	return result
}
