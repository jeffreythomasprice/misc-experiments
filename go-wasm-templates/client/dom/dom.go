package dom

import (
	"fmt"
	"reflect"
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

func MultiQuerySelector(dst interface{}) error {
	// TODO see if .Elem() is always required
	value := reflect.ValueOf(dst).Elem()
	typ := value.Type()
	for i := 0; i < typ.NumField(); i++ {
		f := typ.Field(i)
		if !f.IsExported() {
			continue
		}
		tag := f.Tag.Get("selector")
		if len(tag) == 0 {
			continue
		}
		/*
			TODO different behavior based on type of field
			js.Value = QuerySelector
			[]js.Value = QuerySelectorAll
			other things that can cast into js.Value?
		*/
		elem, err := QuerySelector(tag)
		if err != nil {
			return err
		}
		value.Field(i).Set(reflect.ValueOf(elem))
	}
	return nil
}
