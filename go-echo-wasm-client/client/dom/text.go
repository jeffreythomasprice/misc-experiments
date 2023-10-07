package dom

import (
	"fmt"
	"syscall/js"
)

type text struct {
	value string
}

var _ Renderer = (*text)(nil)

func Text(value string) Renderer {
	return &text{value}
}

func Textf(format string, args ...any) Renderer {
	return Text(fmt.Sprintf(format, args...))
}

// apply implements Renderer.
func (t *text) apply(target *Element) error {
	target.Call("appendChild", js.Global().Get("document").Call("createTextNode", t.value))
	return nil
}
