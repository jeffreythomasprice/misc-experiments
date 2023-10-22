package main

import (
	"client/swap"
	"shared"
	"syscall/js"
)

func main() {
	shared.InitLogging()

	clickCount := 0
	shared.Must0(swap.Swap(
		hello(),
		map[string]any{
			"click": js.FuncOf(func(this js.Value, args []js.Value) any {
				clickCount++
				shared.Must0(swap.Swap(
					clicks(clickCount),
					nil,
					"#clickResults",
					swap.InnerHTML,
				))
				return nil
			}),
		},
		"body",
		swap.InnerHTML,
	))

	select {}
}
