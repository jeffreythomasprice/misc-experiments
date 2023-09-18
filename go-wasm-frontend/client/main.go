package main

import (
	"client/dom"
	"shared"
)

func main() {
	shared.InitSlog()

	document := dom.NewDocument()
	body := document.Body()

	elem := document.CreateElement("h1")
	elem.SetInnerText("Hello, World!")
	replaceContent(body.Element, elem.Node)

	appendContent(body.Element, renderDomString(`
		<ul>
			<li>foo</li>
			<li>bar</li>
			<li>baz</li>
		<ul>
	`)...)

	go liveReload()

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
