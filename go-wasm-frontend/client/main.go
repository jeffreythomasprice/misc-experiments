package main

import (
	"client/dom"
	"html/template"
	"log/slog"
	"shared"
	"strings"
)

func main() {
	shared.InitSlog()

	go liveReload()

	document := dom.NewDocument()
	body := document.Body()

	elem := document.CreateElement("h1")
	elem.SetInnerText("Hello, World!")
	replaceContent(body.Element, elem.Node)

	replaceContent(body.Element, renderDomString(`
		<form id="form">
			<label>Enter a name:</label>
			<input type="text" name="name" placeholder="Name"/>
		</form>
	`)...)
	// TODO simpler casting?
	form := dom.NewHTMLFormElement(*document.QuerySelector("#form").Value)
	// TODO event for when input becomes visible, set focus because autofocus doesn't work when swapping in
	form.OnSubmit(func(e *dom.SubmitEvent) {
		e.PreventDefault()

		slog.Debug("TODO JEFF submit", "entries", e.FormData().Entries())
	})

	select {}
}

func replaceContent(target *dom.Element, newContent ...*dom.Node) {
	for _, child := range target.Children() {
		target.RemoveChild(child)
	}
	appendContent(target, newContent...)
}

func appendContent(target *dom.Element, newContent ...*dom.Node) {
	for _, child := range newContent {
		target.AppendChild(child)
	}
}

func renderDomString(s string) []*dom.Node {
	temp := dom.NewDocument().CreateElement("div")
	temp.SetInnerHTML(s)
	return temp.Children()
}

func newDomRenderer(f func() (string, error)) func() []*dom.Node {
	return func() []*dom.Node {
		s, err := f()
		if err != nil {
			// TODO handle error when rendering
			slog.Error("error rendering dom elements from string", "err", err)
			return nil
		}
		return renderDomString(s)
	}
}

func newTemplateRenderer(t *template.Template, name string, data func() any) func() []*dom.Node {
	return newDomRenderer(func() (string, error) {
		var s strings.Builder
		if err := t.ExecuteTemplate(&s, name, data()); err != nil {
			return "", nil
		}
		return s.String(), nil
	})
}

func newReplaceChildWith(target *dom.Element, f func() []*dom.Node) func() {
	return func() {
		replaceContent(target, f()...)
	}
}
