package main

import (
	"syscall/js"
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

	context := NewWebGl2RenderingContext(canvas)

	shader, err := NewShader(
		context,
		`
		attribute vec2 positionAttribute;

		void main() {
			gl_Position = vec4(positionAttribute, 0, 1);
		}
		`,
		`
		void main() {
			gl_FragColor = vec4(1, 1, 1, 1);
		}
		`,
	)
	if err != nil {
		panic(err)
	}

	arrayBuffer := context.CreateBuffer()
	context.BindBuffer(context.ARRAY_BUFFER(), arrayBuffer)
	context.BufferDataF32(
		context.ARRAY_BUFFER(), []float32{
			-0.5, -0.5,
			0.5, -0.5,
			0.0, 0.5,
		},
		context.STATIC_DRAW(),
	)
	context.BindBuffer(context.ARRAY_BUFFER(), js.Null())

	resize := func() {
		width := window.InnerWidth()
		height := window.InnerHeight()
		canvas.SetWidth(width)
		canvas.SetHeight(height)
		context.Viewport(0, 0, width, height)
	}
	window.AddEventListener("resize", false, func(e dom.Event) {
		resize()
	})
	resize()

	var anim func(time.Duration)
	anim = func(_d time.Duration) {
		context.ClearColor(0.25, 0.5, 0.75, 1.0)
		context.Clear(context.COLOR_BUFFER_BIT())

		shader.UseProgram()

		positionAttribute, err := shader.GetAttribLocation("positionAttribute")
		if err != nil {
			panic(err)
		}
		context.BindBuffer(context.ARRAY_BUFFER(), arrayBuffer)
		context.EnableVertexAttribArray(positionAttribute)
		context.VertexAttribPointer(
			positionAttribute,
			2,
			context.FLOAT(),
			false,
			0,
			0,
		)
		context.DrawArrays(context.TRIANGLES(), 0, 3)
		context.DisableVertexAttribArray(positionAttribute)
		context.BindBuffer(context.ARRAY_BUFFER(), js.Null())

		context.UseProgram(js.Null())

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
