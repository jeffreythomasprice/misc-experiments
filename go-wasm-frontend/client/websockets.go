package main

import (
	"context"
	"errors"
	"log/slog"
	"sync"
	"syscall/js"
	"time"
)

var ErrWebsocket error = errors.New("websocket closed with error")

type WebsocketMessagehandler = func(message string)

func WebsocketWithReconnect(ctx context.Context, url string, protocol []string, onMessage WebsocketMessagehandler) error {
	const initialTime = time.Microsecond * 100
	const maxTime = time.Second * 10
	sleep := initialTime
	for {
		err := Websocket(ctx, url, protocol, onMessage)
		if errors.Is(err, context.Canceled) {
			return err
		}
		if err == nil {
			sleep = initialTime
		} else {
			slog.Error("websocket failure, restarting", "err", err)
			sleep = min(sleep*2, maxTime)
		}
		time.Sleep(sleep)
	}
}

func Websocket(ctx context.Context, url string, protocol []string, onMessage WebsocketMessagehandler) error {
	ws := js.Global().Get("WebSocket").New(url)

	var wg sync.WaitGroup
	wg.Add(1)

	var returnErr error

	go func() {
		<-ctx.Done()
		if returnErr != nil {
			err := ctx.Err()
			if err != nil {
				returnErr = err
			}
		}
		wg.Done()
	}()

	ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Debug("websocket connected", "url", url)
		return nil
	}))

	ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
		wg.Done()
		return nil
	}))

	ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Warn("websocket error", "url", url)
		returnErr = ErrWebsocket
		return nil
	}))

	ws.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
		message := args[0].Get("data")
		if message.Type() == js.TypeString {
			message := message.String()
			onMessage(message)
		} else if message.InstanceOf(js.Global().Get("ArrayBuffer")) {
			panic("TODO JEFF convert array buffer to message")
		} else if message.InstanceOf(js.Global().Get("Blob")) {
			_, err := await(message.Call("arrayBuffer"))
			if err.Truthy() {
				slog.Error("error getting buffer from blob message", "url", url, "err", err)
				returnErr = ErrWebsocket
			} else {
				panic("TODO JEFF convert array buffer to message")
			}
		} else {
			slog.Error("unhandled message type", "url", url, "message", message.Type())
			returnErr = ErrWebsocket
		}
		return nil
	}))

	wg.Wait()
	return returnErr
}

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
