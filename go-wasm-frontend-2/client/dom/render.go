package dom

import (
	"syscall/js"
)

type Rendered []js.Value

type DomRendererFunc = func() (Rendered, error)

func DomString(s string) (Rendered, error) {
	temp := js.Global().Get("document").Call("createElement", "div")
	temp.Set("innerHTML", s)
	tempChildren := temp.Get("children")
	results := make(Rendered, 0)
	for tempChildren.Length() > 0 {
		child := tempChildren.Index(0)
		results = append(results, child)
		temp.Call("removeChild", child)
	}
	return results, nil
}

func Append(selector Selector, children DomRendererFunc) error {
	return commonDomEditsWithSelecctor(selector, children, func(target js.Value, children Rendered, childrenAsAny []any) error {
		target.Call("append", childrenAsAny...)
		return nil
	})
}

func Replace(selector Selector, children DomRendererFunc) error {
	return commonDomEditsWithSelecctor(selector, children, func(target js.Value, children Rendered, childrenAsAny []any) error {
		target.Call("replaceWith", childrenAsAny...)
		return nil
	})
}

func commonDomEditsWithSelecctor(
	selector Selector,
	children DomRendererFunc,
	f func(target js.Value, children Rendered, childrenAsAny []any) error,
) error {
	target, err := selector()
	if err != nil {
		return err
	}

	childElements, err := children()
	if err != nil {
		return err
	}

	childElementsAsAnys := make([]any, 0, len(childElements))
	for _, x := range childElements {
		childElementsAsAnys = append(childElementsAsAnys, x)
	}

	f(target, childElements, childElementsAsAnys)

	return nil
}
