package main

import (
	"syscall/js"
	"unsafe"

	"honnef.co/go/js/dom/v2"
)

type WebGl2RenderingContext struct {
	js.Value
}

func NewWebGl2RenderingContext(canvas *dom.HTMLCanvasElement) *WebGl2RenderingContext {
	return &WebGl2RenderingContext{
		// TODO options, e.g. performance mode
		// TODO check for failures
		canvas.GetContext("webgl2"),
	}
}

func (c *WebGl2RenderingContext) COLOR_BUFFER_BIT() int {
	return c.Get("COLOR_BUFFER_BIT").Int()
}

func (c *WebGl2RenderingContext) VERTEX_SHADER() int {
	return c.Get("VERTEX_SHADER").Int()
}

func (c *WebGl2RenderingContext) FRAGMENT_SHADER() int {
	return c.Get("FRAGMENT_SHADER").Int()
}

func (c *WebGl2RenderingContext) COMPILE_STATUS() int {
	return c.Get("COMPILE_STATUS").Int()
}

func (c *WebGl2RenderingContext) LINK_STATUS() int {
	return c.Get("LINK_STATUS").Int()
}

func (c *WebGl2RenderingContext) ARRAY_BUFFER() int {
	return c.Get("ARRAY_BUFFER").Int()
}

func (c *WebGl2RenderingContext) ELEMENT_ARRAY_BUFFER() int {
	return c.Get("ELEMENT_ARRAY_BUFFER").Int()
}

func (c *WebGl2RenderingContext) STATIC_DRAW() int {
	return c.Get("STATIC_DRAW").Int()
}

func (c *WebGl2RenderingContext) DYNAMIC_DRAW() int {
	return c.Get("DYNAMIC_DRAW").Int()
}

func (c *WebGl2RenderingContext) STREAM_DRAW() int {
	return c.Get("STREAM_DRAW").Int()
}

func (c *WebGl2RenderingContext) STATIC_READ() int {
	return c.Get("STATIC_READ").Int()
}

func (c *WebGl2RenderingContext) DYNAMIC_READ() int {
	return c.Get("DYNAMIC_READ").Int()
}

func (c *WebGl2RenderingContext) STREAM_READ() int {
	return c.Get("STREAM_READ").Int()
}

func (c *WebGl2RenderingContext) STATIC_COPY() int {
	return c.Get("STATIC_COPY").Int()
}

func (c *WebGl2RenderingContext) DYNAMIC_COPY() int {
	return c.Get("DYNAMIC_COPY").Int()
}

func (c *WebGl2RenderingContext) STREAM_COPY() int {
	return c.Get("STREAM_COPY").Int()
}

func (c *WebGl2RenderingContext) FLOAT() int {
	return c.Get("FLOAT").Int()
}

func (c *WebGl2RenderingContext) TRIANGLES() int {
	return c.Get("TRIANGLES").Int()
}

func (c *WebGl2RenderingContext) ClearColor(red, green, blue, alpha float64) {
	c.Call("clearColor", red, green, blue, alpha)
}

func (c *WebGl2RenderingContext) Clear(bits int) {
	c.Call("clear", bits)
}

func (c *WebGl2RenderingContext) Viewport(x, y, width, height int) {
	c.Call("viewport", x, y, width, height)
}

func (c *WebGl2RenderingContext) CreateShader(typ int) js.Value {
	return c.Call("createShader", typ)
}

func (c *WebGl2RenderingContext) DeleteShader(shader js.Value) {
	c.Call("deleteShader", shader)
}

func (c *WebGl2RenderingContext) ShaderSource(shader js.Value, source string) {
	c.Call("shaderSource", shader, source)
}

func (c *WebGl2RenderingContext) CompileShader(shader js.Value) {
	c.Call("compileShader", shader)
}

func (c *WebGl2RenderingContext) GetShaderParameter(shader js.Value, param int) js.Value {
	return c.Call("getShaderParameter", shader, param)
}

func (c *WebGl2RenderingContext) GetShaderInfoLog(shader js.Value) string {
	return c.Call("getShaderInfoLog", shader).String()
}

func (c *WebGl2RenderingContext) CreateProgram() js.Value {
	return c.Call("createProgram")
}

func (c *WebGl2RenderingContext) DeleteProgram(program js.Value) {
	c.Call("deleteProgram", program)
}

func (c *WebGl2RenderingContext) AttachShader(program, shader js.Value) {
	c.Call("attachShader", program, shader)
}

func (c *WebGl2RenderingContext) DetachShader(program, shader js.Value) {
	c.Call("detachShader", program, shader)
}

func (c *WebGl2RenderingContext) LinkProgram(program js.Value) {
	c.Call("linkProgram", program)
}

func (c *WebGl2RenderingContext) GetProgramParameter(program js.Value, param int) js.Value {
	return c.Call("getProgramParameter", program, param)
}

func (c *WebGl2RenderingContext) GetProgramInfoLog(program js.Value) string {
	return c.Call("getProgramInfoLog", program).String()
}

func (c *WebGl2RenderingContext) GetAttribLocation(program js.Value, name string) int {
	return c.Call("getAttribLocation", program, name).Int()
}

// TODO various shader methods for getting attributes and uniforms

func (c *WebGl2RenderingContext) UseProgram(program js.Value) {
	c.Call("useProgram", program)
}

func (c *WebGl2RenderingContext) CreateBuffer() js.Value {
	return c.Call("createBuffer")
}

func (c *WebGl2RenderingContext) DeleteBuffer(buffer js.Value) {
	c.Call("deleteBuffer", buffer)
}

func (c *WebGl2RenderingContext) BindBuffer(typ int, buffer js.Value) {
	c.Call("bindBuffer", typ, buffer)
}

// TODO BufferDataSize
// TODO BufferDataU8
// TODO BufferDataF64

func (c *WebGl2RenderingContext) BufferDataF32(typ int, data []float32, usage int) {
	// TODO cache temp?
	const bytesPerElement = 4
	tmp := js.Global().Get("Uint8Array").New(len(data) * bytesPerElement)
	js.CopyBytesToJS(tmp, unsafe.Slice((*byte)(unsafe.Pointer(&data[0])), len(data)*bytesPerElement))
	c.Call("bufferData", typ, tmp, usage)
}

func (c *WebGl2RenderingContext) EnableVertexAttribArray(i int) {
	c.Call("enableVertexAttribArray", i)
}

func (c *WebGl2RenderingContext) DisableVertexAttribArray(i int) {
	c.Call("disableVertexAttribArray", i)
}

func (c *WebGl2RenderingContext) VertexAttribPointer(index, size, typ int, normalized bool, stride, offset int) {
	c.Call("vertexAttribPointer", index, size, typ, normalized, stride, offset)
}

func (c *WebGl2RenderingContext) DrawArrays(mode, first, count int) {
	c.Call("drawArrays", mode, first, count)
}

// TODO drawElements
