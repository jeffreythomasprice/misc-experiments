package dom

func Button(renderers ...Renderer) *Element {
	return NewElement("button", renderers...)
}

func Div(renderers ...Renderer) *Element {
	return NewElement("div", renderers...)
}

func Form(renderers ...Renderer) *Element {
	return NewElement("form", renderers...)
}

func H1(renderers ...Renderer) *Element {
	return NewElement("h1", renderers...)
}

func H2(renderers ...Renderer) *Element {
	return NewElement("h2", renderers...)
}

func H3(renderers ...Renderer) *Element {
	return NewElement("h3", renderers...)
}

func H4(renderers ...Renderer) *Element {
	return NewElement("h4", renderers...)
}

func H5(renderers ...Renderer) *Element {
	return NewElement("h5", renderers...)
}

func H6(renderers ...Renderer) *Element {
	return NewElement("h6", renderers...)
}

func Input(renderers ...Renderer) *Element {
	return NewElement("input", renderers...)
}

func Label(renderers ...Renderer) *Element {
	return NewElement("label", renderers...)
}

func P(renderers ...Renderer) *Element {
	return NewElement("p", renderers...)
}
