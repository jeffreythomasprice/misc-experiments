package main

import (
	"sync"
	"syscall/js"
)

func await(promise js.Value) (result js.Value, err js.Value) {
	var wg sync.WaitGroup
	wg.Add(1)
	promise.Call("then", js.FuncOf(func(this js.Value, args []js.Value) any {
		result = args[0]
		err = js.Null()
		wg.Done()
		return nil
	}))
	promise.Call("catch", js.FuncOf(func(this js.Value, args []js.Value) any {
		result = js.Null()
		err = args[0]
		wg.Done()
		return nil
	}))
	wg.Wait()
	return
}
