package dom

type attr struct {
	name, value string
}

var _ Renderer = (*attr)(nil)

func Attr(name, value string) Renderer {
	return &attr{name, value}
}

// apply implements Renderer.
func (a *attr) apply(target *Element) error {
	target.Set(a.name, a.value)
	return nil
}
