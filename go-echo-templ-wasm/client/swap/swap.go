package swap

import (
	"context"
	"fmt"
	"strings"
	"syscall/js"

	"github.com/a-h/templ"
)

type SwapOperation func(targetElement js.Value, newChildren []js.Value) error

/*
Swap applies some dynamic update function to the dom, creating new elements and potentially removing unneeded ones.

New elements are created by inserting the given component into the dom via setting a temporary element's innerHTML.

Any attributes on the new elements where the value matches a key in the dynamic attributes map are replaced by the values from that map.
Values in the map must be either js.Value or things that you can call js.ValueOf on.

The target selectors are evaluated by calling document.querySelector. If no element is found an error is returned.

The method for inserting the resulting elements into the dom is determined by the swap operation. e.g. InnerHTML replaces any content at the
selected element with the new elements.
*/
func Swap(component templ.Component, dynamicAttributes map[string]any, targetSelectors string, operation SwapOperation) error {
	// find the target
	document := js.Global().Get("document")
	targetElement := document.Call("querySelector", targetSelectors)
	if !targetElement.Truthy() {
		return fmt.Errorf("failed to find target: %v", targetSelectors)
	}

	// render to string
	var componentString strings.Builder
	if err := component.Render(context.Background(), &componentString); err != nil {
		return fmt.Errorf("error rendering component for swap: %w", err)
	}

	// turn string into dom elements
	tempDiv := document.Call("createElement", "div")
	tempDiv.Set("innerHTML", componentString.String())
	tempChildren := tempDiv.Get("children")
	children := make([]js.Value, 0, tempChildren.Length())
	for tempChildren.Length() > 0 {
		child := tempChildren.Index(0)
		tempDiv.Call("removeChild", child)
		children = append(children, child)
	}

	// eval all dynamic attributes
	if dynamicAttributes != nil {
		evalProps := func(elem js.Value) error {
			attrs := elem.Get("attributes")
			for i := 0; i < attrs.Length(); i++ {
				attr := attrs.Index(i)
				name := attr.Get("name").String()
				value := attr.Get("value").String()
				dyn, exists := dynamicAttributes[value]
				if exists {
					dynValue, err := safeJsValueOf(dyn)
					if err != nil {
						return fmt.Errorf("failed to set dynamic attribute %v=%v: %w", name, value, err)
					}
					elem.Set(name, dynValue)
				}
			}
			return nil
		}
		for _, child := range children {
			if err := evalProps(child); err != nil {
				return err
			}
		}
	}

	// perform the swap operation on the target
	if err := operation(targetElement, children); err != nil {
		return fmt.Errorf("failed to perform swap operation: %w", err)
	}

	return nil
}

func InnerHTML(targetElement js.Value, newChildren []js.Value) error {
	asAny := make([]any, 0, len(newChildren))
	for _, x := range newChildren {
		asAny = append(asAny, x)
	}
	targetElement.Call("replaceChildren", asAny...)
	return nil
}

func safeJsValueOf(x any) (result js.Value, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("failed to convert %v to js.Value: %v", x, r)
		}
	}()
	result = js.ValueOf(x)
	return
}
