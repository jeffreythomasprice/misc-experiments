package dom

import "syscall/js"

type text struct {
	value string
}

var _ Renderer = (*text)(nil)

func Text(value string) Renderer {
	return &text{value}
}

// apply implements Renderer.
func (t *text) apply(target *Element) error {
	target.Call("appendChild", js.Global().Get("document").Call("createTextNode", t.value))
	return nil
}
