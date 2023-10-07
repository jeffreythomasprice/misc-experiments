package dom

type class struct {
	name string
}

var _ Renderer = (*class)(nil)

func Class(name string) Renderer {
	return &class{name}
}

// apply implements Renderer.
func (c *class) apply(target *Element) error {
	target.Get("classList").Call("add", c.name)
	return nil
}
