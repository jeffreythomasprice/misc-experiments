package dom

type Renderer interface {
	apply(target *Element) error
}
