package main

import (
	"github.com/Zyko0/go-sdl3/bin/binsdl"
	"github.com/Zyko0/go-sdl3/sdl"
)

func main() {
	defer binsdl.Load().Unload()
	defer sdl.Quit()

	if err := sdl.Init(sdl.INIT_VIDEO); err != nil {
		panic(err)
	}

	window, renderer, err := sdl.CreateWindowAndRenderer("Hello world", 1024, 768, sdl.WINDOW_OPENGL)
	if err != nil {
		panic(err)
	}
	defer renderer.Destroy()
	defer window.Destroy()

	renderer.SetDrawColor(255, 255, 255, 255)

	sdl.RunLoop(func() error {
		var event sdl.Event

		for sdl.PollEvent(&event) {
			if event.Type == sdl.EVENT_QUIT {
				return sdl.EndLoop
			}
			if event.Type == sdl.EVENT_KEY_UP && event.KeyboardEvent().Scancode == sdl.SCANCODE_ESCAPE {
				return sdl.EndLoop
			}
		}

		windowWidth, windowHeight, err := window.Size()
		if err != nil {
			panic(err)
		}

		renderer.SetDrawColor(64, 64, 64, 255)
		renderer.Clear()

		renderer.RenderGeometry(
			nil,
			[]sdl.Vertex{
				{
					Position: sdl.FPoint{
						X: float32(windowWidth) * 0.25,
						Y: float32(windowHeight) * 0.25,
					},
					Color:    sdl.FColor{R: 1.0, G: 0.0, B: 0.0, A: 1.0},
					TexCoord: sdl.FPoint{},
				},
				{
					Position: sdl.FPoint{
						X: float32(windowWidth) * 0.75,
						Y: float32(windowHeight) * 0.25,
					},
					Color:    sdl.FColor{R: 0.0, G: 1.0, B: 0.0, A: 1.0},
					TexCoord: sdl.FPoint{},
				},
				{
					Position: sdl.FPoint{
						X: float32(windowWidth) * 0.75,
						Y: float32(windowHeight) * 0.75,
					},
					Color:    sdl.FColor{R: 0.0, G: 0.0, B: 1.0, A: 1.0},
					TexCoord: sdl.FPoint{},
				},
				{
					Position: sdl.FPoint{
						X: float32(windowWidth) * 0.25,
						Y: float32(windowHeight) * 0.75,
					},
					Color:    sdl.FColor{R: 1.0, G: 0.0, B: 1.0, A: 1.0},
					TexCoord: sdl.FPoint{},
				},
			},
			[]int32{
				0, 1, 2,
				2, 3, 0,
			},
		)

		renderer.DebugText(50, 50, "Hello world")
		renderer.Present()

		return nil
	})
}
