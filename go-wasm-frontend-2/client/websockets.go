package main

import (
	"context"
	"log/slog"
	"syscall/js"
	"time"
)

type WebsocketEvent interface{}

type WebsocketOpenEvent struct{}

var _ WebsocketEvent = WebsocketOpenEvent{}

type WebsocketClosedEvent struct{}

var _ WebsocketEvent = WebsocketClosedEvent{}

type WebsocketErrorEvent struct{}

var _ WebsocketEvent = WebsocketErrorEvent{}

type WebsocketTextMessageEvent struct {
	data string
}

var _ WebsocketEvent = WebsocketTextMessageEvent{}

type WebsocketBinaryMessageEvent struct {
	data []byte
}

var _ WebsocketEvent = WebsocketBinaryMessageEvent{}

func (e WebsocketTextMessageEvent) Data() string {
	return e.data
}

func (e WebsocketBinaryMessageEvent) Data() []byte {
	return e.data
}

type WebsocketClientOptions struct {
	Protocol  []string
	Reconnect bool
}

func NewWebsocket(ctx context.Context, url string, options *WebsocketClientOptions) (read <-chan WebsocketEvent, write chan<- WebsocketEvent) {
	resultRead := make(chan WebsocketEvent)
	resultWrite := make(chan WebsocketEvent)

	doOnce := func() {
		currentRead, currentWrite := newWebsocket(ctx, url, options.Protocol)

		go func() {
			for msg := range resultWrite {
				currentWrite <- msg
			}
		}()

		for msg := range currentRead {
			resultRead <- msg
		}
	}

	if options.Reconnect {
		go func() {
			const initialTime = time.Millisecond * 10
			const maxTime = time.Second * 5
			sleep := initialTime

			for {
				doOnce()

				select {
				case <-ctx.Done():
					return
				default:
				}
				sleep = min(sleep*2, maxTime)
				time.Sleep(sleep)
			}
		}()
	} else {
		doOnce()
	}

	read = resultRead
	write = resultWrite
	return
}

func newWebsocket(ctx context.Context, url string, protocol []string) (read <-chan WebsocketEvent, write chan<- WebsocketEvent) {
	resultRead := make(chan WebsocketEvent)
	resultWrite := make(chan WebsocketEvent)

	var ws js.Value
	if protocol != nil {
		ws = js.Global().Get("WebSocket").New(url, protocol)
	} else {
		ws = js.Global().Get("WebSocket").New(url)
	}

	// TODO respect ctx

	go func() {
		for msg := range resultRead {
			switch typedMsg := msg.(type) {
			case WebsocketTextMessageEvent:
				ws.Call("send", typedMsg.Data())
			case WebsocketBinaryMessageEvent:
				panic("TODO write binary data")
			}
		}
	}()

	ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Debug("websocket connected", "url", url)
		resultRead <- WebsocketOpenEvent{}
		return nil
	}))

	ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Debug("websocket close", "url", url)
		resultRead <- WebsocketClosedEvent{}
		close(resultRead)
		close(resultWrite)
		return nil
	}))

	ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Warn("websocket error", "url", url)
		resultRead <- WebsocketErrorEvent{}
		ws.Call("close")
		return nil
	}))

	ws.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
		message := args[0].Get("data")
		if message.Type() == js.TypeString {
			message := message.String()
			resultRead <- WebsocketTextMessageEvent{data: message}
		} else if message.InstanceOf(js.Global().Get("ArrayBuffer")) {
			panic("TODO convert array buffer to message")
		} else if message.InstanceOf(js.Global().Get("Blob")) {
			_, err := await(message.Call("arrayBuffer"))
			if err.Truthy() {
				slog.Error("error getting buffer from blob message", "url", url, "err", err)
			} else {
				panic("TODO convert array buffer to message")
			}
		} else {
			slog.Error("unhandled message type", "url", url, "message", message.Type())
		}
		return nil
	}))

	read = resultRead
	write = resultWrite
	return
}
