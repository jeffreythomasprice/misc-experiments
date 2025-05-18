package main

import (
	"fmt"
	"math"

	"github.com/Zyko0/go-sdl3/bin/binsdl"
	"github.com/Zyko0/go-sdl3/sdl"
)

func main() {
	if err := run(); err != nil {
		panic(err)
	}
}

func run() error {
	defer binsdl.Load().Unload()
	defer sdl.Quit()

	if err := sdl.Init(sdl.INIT_VIDEO); err != nil {
		return fmt.Errorf("error initializing sdl: %w", err)
	}

	window, renderer, err := sdl.CreateWindowAndRenderer(
		"Experiment",
		1024,
		768,
		sdl.WINDOW_OPENGL|sdl.WINDOW_RESIZABLE,
	)
	if err != nil {
		return fmt.Errorf("error creating sdl window and renderer: %w", err)
	}
	defer renderer.Destroy()
	defer window.Destroy()

	if err := sdl.RunLoop(func() error {
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
			return fmt.Errorf("error getting window size: %w", err)
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

		{
			points, indices, err := createCircleTriangles(
				sdl.FPoint{
					X: float32(windowWidth) * 0.5,
					Y: float32(windowHeight) * 0.5,
				},
				150.0,
				32,
			)
			if err != nil {
				return fmt.Errorf("error creating points for circle: %w", err)
			}
			vertices := createSolidColorUntexturedVerticesFromPoints(points, sdl.FColor{R: 0.75, G: 0.75, B: 0.75, A: 1})
			renderer.RenderGeometry(nil, vertices, indices)
		}

		renderer.SetDrawColor(255, 255, 255, 255)
		renderer.DebugText(50, 50, "Hello world")
		renderer.Present()

		return nil
	}); err != nil {
		return fmt.Errorf("error in sdl run loop: %w", err)
	}

	return nil
}

func createCircleTriangles(center sdl.FPoint, radius float32, numPoints int) ([]sdl.FPoint, []int32, error) {
	if numPoints < 3 {
		return nil, nil, fmt.Errorf("must have at least 3 points on the edge of the circle, got %v", numPoints)
	}
	angleStep := math.Pi * 2 / float64(numPoints)
	angle := 0.0
	points := make([]sdl.FPoint, 0, numPoints+1)
	indices := make([]int32, 0, 3*(numPoints-2))
	points = append(points, center)
	for range numPoints {
		points = append(points, sdl.FPoint{
			X: float32(math.Cos(angle))*radius + center.X,
			Y: float32(math.Sin(angle))*radius + center.Y,
		})
		angle += angleStep
	}
	for i := range numPoints {
		j := (i + 1) % numPoints
		i++
		j++
		indices = append(indices, 0, int32(i), int32(j))
	}
	return points, indices, nil
}

func createSolidColorUntexturedVerticesFromPoints(points []sdl.FPoint, color sdl.FColor) []sdl.Vertex {
	result := make([]sdl.Vertex, 0, len(points))
	for _, p := range points {
		result = append(result, sdl.Vertex{
			Position: p,
			Color:    color,
		})
	}
	return result
}
