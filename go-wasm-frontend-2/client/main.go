package main

import (
	"errors"
	"fmt"
	"log/slog"
	"shared"
	"syscall/js"
)

func main() {
	shared.InitSlog()

	go liveReload("ws://localhost:8000/_liveReload")

	must(Append("body", func() ([]js.Value, error) {
		return DomString(`
			<div id="message">Hello, World!</div>
			<button>Click Me</button>
		`)
	}))

	count := 0
	must(OnClick("button", func(e js.Value) {
		slog.Debug("clicked!")

		must(Replace("#message", func() ([]js.Value, error) {
			count++
			return DomString(fmt.Sprintf(`
				<div id="message">%v</div>
			`, count))
		}))
	}))

	select {}
}

var ErrSelectorNotFound = errors.New("no such element for selector")

type DomRendererFunc = func() ([]js.Value, error)

func Append(selectors string, children DomRendererFunc) error {
	return commonDomEditsWithSelecctor(selectors, children, func(target js.Value, children []js.Value, childrenAsAny []any) error {
		target.Call("append", childrenAsAny...)
		return nil
	})
}

func Replace(selectors string, children DomRendererFunc) error {
	return commonDomEditsWithSelecctor(selectors, children, func(target js.Value, children []js.Value, childrenAsAny []any) error {
		target.Call("replaceWith", childrenAsAny...)
		return nil
	})
}

func DomString(s string) ([]js.Value, error) {
	temp := js.Global().Get("document").Call("createElement", "div")
	temp.Set("innerHTML", s)
	tempChildren := temp.Get("children")
	results := make([]js.Value, 0)
	for tempChildren.Length() > 0 {
		child := tempChildren.Index(0)
		results = append(results, child)
		temp.Call("removeChild", child)
	}
	return results, nil
}

func AddEventListener(selectors, eventName string, f func(args []js.Value)) error {
	target, err := querySelector(selectors)
	if err != nil {
		return err
	}

	target.Call("addEventListener", eventName, js.FuncOf(func(this js.Value, args []js.Value) any {
		f(args)
		return nil
	}))

	return nil
}

func OnClick(selectors string, f func(e js.Value)) error {
	return AddEventListener(selectors, "click", func(args []js.Value) {
		f(args[0])
	})
}

func commonDomEditsWithSelecctor(
	selectors string,
	children DomRendererFunc,
	f func(target js.Value, children []js.Value, childrenAsAny []any) error,
) error {
	target, err := querySelector(selectors)
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

func querySelector(selectors string) (js.Value, error) {
	result := js.Global().Get("document").Call("querySelector", selectors)
	if !result.Truthy() {
		return js.Null(), fmt.Errorf("%w, was looking for %v", ErrSelectorNotFound, selectors)
	}
	return result, nil
}

func must(err error) {
	if err != nil {
		panic(err)
	}
}
