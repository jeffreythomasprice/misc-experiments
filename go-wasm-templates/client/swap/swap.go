package swap

import (
	"fmt"
	"io"
	"strings"
	"syscall/js"
)

type Operation func(target js.Value, newElements []js.Value) error

type Generator func(w io.Writer) error

type EventHandler func(this js.Value, args []js.Value)

func Swap(
	selectors string,
	op Operation,
	gen Generator,
	events map[string]EventHandler,
) error {
	target, err := querySelector(selectors)
	if err != nil {
		return err
	}

	elements, err := render(gen)
	if err != nil {
		return err
	}

	err = op(target, elements)
	if err != nil {
		return err
	}

	for _, e := range elements {
		if err := iterateAllAttributes(e, func(element js.Value, key, value string) error {
			if strings.HasPrefix(key, "go-") {
				realKey := strings.TrimPrefix(key, "go-")
				e.Get("attributes").Call("removeNamedItem", key)
				event, ok := events[value]
				if !ok {
					return fmt.Errorf("no such event: %v, was trying to handle %v", value, key)
				}
				element.Call("addEventListener", realKey, js.FuncOf(func(this js.Value, args []js.Value) any {
					event(this, args)
					return nil
				}))
			}
			return nil
		}); err != nil {
			return err
		}
	}

	return nil
}

var InnerHTML Operation

// TODO more operations: OuterHTML, Append, Prepend, AppendSibling, PrependSibling

func init() {
	InnerHTML = func(target js.Value, newElements []js.Value) error {
		newElementsAsAny := make([]any, 0, len(newElements))
		for _, x := range newElements {
			newElementsAsAny = append(newElementsAsAny, x)
		}
		target.Call("replaceChildren", newElementsAsAny...)
		return nil
	}
}

func querySelector(selectors string) (js.Value, error) {
	result := js.Global().Get("document").Call("querySelector", selectors)
	if !result.Truthy() {
		return js.Null(), fmt.Errorf("no such element for selectors: %v", selectors)
	}
	return result, nil
}

func render(gen Generator) ([]js.Value, error) {
	var s strings.Builder
	if err := gen(&s); err != nil {
		return nil, err
	}

	temp := js.Global().Get("document").Call("createElement", "div")
	temp.Set("innerHTML", s.String())

	children := temp.Get("childNodes")
	if !children.Truthy() {
		return nil, nil
	}
	len := children.Length()

	results := make([]js.Value, 0, len)
	for i := 0; i < len; i++ {
		results = append(results, children.Index(i))
	}

	temp.Call("replaceChildren")

	return results, nil
}

func iterateAllAttributes(element js.Value, f func(element js.Value, key, value string) error) error {
	attrs := element.Get("attributes")
	if attrs.Truthy() {
		len := attrs.Length()
		for i := 0; i < len; i++ {
			item := attrs.Call("item", i)
			if err := f(element, item.Get("name").String(), item.Get("value").String()); err != nil {
				return err
			}
		}
	}

	children := element.Get("children")
	if children.Truthy() {
		len := children.Length()
		for i := 0; i < len; i++ {
			if err := iterateAllAttributes(children.Index(i), f); err != nil {
				return err
			}
		}
	}

	return nil
}
