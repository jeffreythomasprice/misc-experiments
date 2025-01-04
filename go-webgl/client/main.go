package main

import (
	"time"

	"honnef.co/go/js/dom/v2"
)

func main() {
	window := dom.GetWindow()
	document := window.Document().(dom.HTMLDocument)
	body := document.Body()
	for _, e := range body.ChildNodes() {
		body.RemoveChild(e)
	}

	canvas := document.CreateElement("canvas").(*dom.HTMLCanvasElement)
	body.AppendChild(canvas)
	canvas.Style().Set("position", "absolute")
	canvas.Style().Set("left", "0")
	canvas.Style().Set("top", "0")
	canvas.Style().Set("width", "100%")
	canvas.Style().Set("height", "100%")

	context := canvas.GetContext("webgl2")

	resize := func() {
		width := window.InnerWidth()
		height := window.InnerHeight()
		canvas.SetWidth(width)
		canvas.SetHeight(height)
		context.Call("viewport", 0, 0, width, height)
	}
	window.AddEventListener("resize", false, func(e dom.Event) {
		resize()
	})
	resize()

	var anim func(time.Duration)
	anim = func(d time.Duration) {
		context.Call("clearColor", 0.25, 0.5, 0.75, 1.0)
		context.Call("clear", context.Get("COLOR_BUFFER_BIT"))

		window.RequestAnimationFrame(func(d time.Duration) {
			anim(d)
		})
	}
	window.RequestAnimationFrame(func(d time.Duration) {
		anim(d)
	})

	// don't let event loop stop
	select {}
}
