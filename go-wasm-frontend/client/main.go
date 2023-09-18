package main

import (
	"client/dom"
	"html/template"
	"log/slog"
	"shared"
	"strings"
	"syscall/js"
)

func main() {
	shared.InitSlog()

	go liveReload()

	document := dom.NewDocument()
	body := document.Body()

	elem := document.CreateElement("h1")
	elem.SetInnerText("Hello, World!")
	replaceContent(body.Element, elem.Node)

	appendContent(body.Element, renderDomString(`
		<div>
			<ul>
				<li>foo</li>
				<li>bar</li>
				<li>baz</li>
			<ul>
		</div>
		<button id="button">Click Me</button>
		<div id="output"></div>
	`)...)

	button := document.QuerySelector("#button")
	output := document.QuerySelector("#output")

	t, err := template.New("").Parse(`
		<h4>count = {{.}}</h4>
	`)
	if err != nil {
		panic(err)
	}
	count := 0
	f := newReplaceChildWith(output, newTemplateRenderer(t, "", func() any {
		count++
		return count
	}))

	button.AddEventListener("click", func(args []js.Value) {
		f()
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

func newTemplateRenderer(t *template.Template, name string, data func() any) func() []*dom.Node {
	return func() []*dom.Node {
		var s strings.Builder
		if err := t.ExecuteTemplate(&s, name, data()); err != nil {
			// TODO handle error when rendering
			slog.Error("error rendering template", "name", name, "err", err)
			return nil
		}
		return renderDomString(s.String())
	}
}

func newReplaceChildWith(target *dom.Element, f func() []*dom.Node) func() {
	return func() {
		replaceContent(target, f()...)
	}
}
