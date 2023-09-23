package main

import (
	"context"
	"encoding/json"
	"errors"
	"log/slog"
	"sync"
	"syscall/js"
	"time"
)

var ErrWebsocket error = errors.New("websocket closed with error")

type WebsocketClient struct {
	ws                    js.Value
	done                  context.Context
	exitErr               error
	isIntentionallyClosed bool
}

type WebsocketMessageHandler = func(message string)

func NewWebsocketWithReconnect(url string, protocol []string, onMessage WebsocketMessageHandler) *WebsocketClient {
	const initialTime = time.Microsecond * 100
	const maxTime = time.Second * 10
	sleep := initialTime

	result := &WebsocketClient{
		done: context.Background(),
	}

	var wg sync.WaitGroup
	wg.Add(1)
	first := true

	go func() {
		for {
			client := NewWebsocket(url, protocol, onMessage)
			result.ws = client.ws
			if first {
				first = false
				wg.Done()
			}

			// wait until the client has connected or failed
			<-client.done.Done()

			// if we've been signalled to stop we're done, stop retrying
			if result.isIntentionallyClosed {
				break
			}

			// restart immediately or after a longer delay, depending on if we failed or not
			if client.exitErr == nil {
				sleep = initialTime
			} else {
				slog.Error("websocket failure, restarting", "err", client.exitErr)
				sleep = min(sleep*2, maxTime)
			}
			time.Sleep(sleep)
		}
	}()

	wg.Wait()
	return result
}

func NewWebsocket(url string, protocol []string, onMessage WebsocketMessageHandler) *WebsocketClient {
	ws := js.Global().Get("WebSocket").New(url)

	connectedOrClosedCtx, connectedOrClosedSignal := context.WithCancel(context.Background())

	done, closed := context.WithCancel(context.Background())

	result := &WebsocketClient{
		ws:      ws,
		done:    done,
		exitErr: nil,
	}

	ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Debug("websocket connected", "url", url)
		// so we can exit the function once we're connected
		connectedOrClosedSignal()
		return nil
	}))

	ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
		closed()
		// in case we need to exit the function without ever getting connected
		connectedOrClosedSignal()
		return nil
	}))

	ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Warn("websocket error", "url", url)
		result.exitErr = ErrWebsocket
		ws.Call("close")
		return nil
	}))

	ws.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
		message := args[0].Get("data")
		if message.Type() == js.TypeString {
			message := message.String()
			onMessage(message)
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

	<-connectedOrClosedCtx.Done()
	return result
}

func (client *WebsocketClient) Close() {
	client.isIntentionallyClosed = true
	client.ws.Call("close")
}

func (client *WebsocketClient) SendString(s string) {
	client.ws.Call("send", s)
}

func (client *WebsocketClient) SendJSON(v interface{}) error {
	bytes, err := json.Marshal(v)
	if err != nil {
		return err
	}
	client.SendString(string(bytes))
	return nil
}
