package websockets

import (
	"errors"
	"fmt"
	"log/slog"
	"strings"
	"sync"
	"syscall/js"
	"time"
)

type OpenFunc func()

type CloseFunc func()

type ErrorFunc func()

type TextMessageFunc func(value string)

type BinaryMessageFunc func(value []byte)

type ReconnectStrategy func(failuresInARow int, lastFailureDelay time.Duration) (retry bool, delayBeforeRetrying time.Duration)

type Builder struct {
	protocols         []string
	openFunc          OpenFunc
	closeFunc         CloseFunc
	errorFunc         ErrorFunc
	textMessageFunc   TextMessageFunc
	binaryMessageFunc BinaryMessageFunc
	reconnectStrategy ReconnectStrategy
}

type Connection struct {
	closed bool
	wg     sync.WaitGroup
	value  js.Value
}

func NewBuilder() *Builder {
	return &Builder{}
}

func (b *Builder) Protocol(protocols ...string) *Builder {
	b.protocols = append(b.protocols, protocols...)
	return b
}

func (b *Builder) OnOpen(f OpenFunc) *Builder {
	b.openFunc = f
	return b
}

func (b *Builder) OnClose(f CloseFunc) *Builder {
	b.closeFunc = f
	return b
}

func (b *Builder) OnError(f ErrorFunc) *Builder {
	b.errorFunc = f
	return b
}

func (b *Builder) OnTextMessage(f TextMessageFunc) *Builder {
	b.textMessageFunc = f
	return b
}

func (b *Builder) OnBinaryMessage(f BinaryMessageFunc) *Builder {
	b.binaryMessageFunc = f
	return b
}

func (b *Builder) ReconnectStrategy(f ReconnectStrategy) *Builder {
	b.reconnectStrategy = f
	return b
}

func (b *Builder) Build(url string) *Connection {
	// if we've been given a relative path guess that they mean the equivalent to what they're looking at for the actual webpage
	// i.e. http => ws and https => wss, same port
	if !strings.HasPrefix(url, "ws://") && !strings.HasPrefix(url, "wss://") {
		wsBaseUrl := js.Global().Get("window").Get("location").Get("origin").String()
		if strings.HasPrefix(wsBaseUrl, "http://") {
			wsBaseUrl = "ws://" + strings.TrimPrefix(wsBaseUrl, "http://")
		} else if strings.HasPrefix(wsBaseUrl, "https://") {
			wsBaseUrl = "wss://" + strings.TrimPrefix(wsBaseUrl, "https://")
		} else {
			panic(fmt.Errorf("unrecognized url prefix: %v", wsBaseUrl))
		}
		newUrl := wsBaseUrl + url
		slog.Debug("url looks like a relative path, prepending assumed websocket base url", "url", url, "wsBaseUrl", wsBaseUrl, "newUrl", newUrl)
		url = newUrl
	}

	args := []any{url}
	if len(b.protocols) > 0 {
		protocols := make([]any, 0, len(b.protocols))
		for _, p := range b.protocols {
			protocols = append(protocols, p)
		}
		args = append(args, protocols)
	}

	openFunc := b.openFunc
	closeFunc := b.closeFunc
	errorFunc := b.errorFunc
	textMessageFunc := b.textMessageFunc
	binaryMessageFunc := b.binaryMessageFunc
	reconnectStrategy := b.reconnectStrategy

	result := &Connection{
		closed: false,
	}

	result.wg.Add(1)
	go func() {
		defer result.wg.Done()

		failuresInARow := 0
		var lastFailureDelay time.Duration = 0
		for {
			result.value = js.Global().Get("WebSocket").New(args...)

			var wg sync.WaitGroup
			wg.Add(1)

			result.value.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket open", "url", url)
				failuresInARow = 0
				if openFunc != nil {
					openFunc()
				}
				return nil
			}))

			result.value.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
				defer wg.Done()

				slog.Debug("websocket close", "url", url)

				result.value = js.Null()

				if closeFunc != nil {
					closeFunc()
				}

				return nil
			}))

			result.value.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
				failuresInARow++
				slog.Debug("websocket error", "url", url, "failuresInARow", failuresInARow)

				if errorFunc != nil {
					errorFunc()
				}
				return nil
			}))

			result.value.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
				data := args[0].Get("data")

				if data.Type() == js.TypeString {
					if textMessageFunc != nil {
						textMessageFunc(data.String())
					}
				} else {
					if binaryMessageFunc != nil {
						handleArrayBuffer := func(arrayBuffer js.Value) {
							bytes := make([]byte, arrayBuffer.Get("byteLength").Int())
							js.CopyBytesToGo(bytes, js.Global().Get("Uint8Array").New(arrayBuffer))
							binaryMessageFunc(bytes)
						}
						if data.InstanceOf(js.Global().Get("Blob")) {
							data.
								Call("arrayBuffer").
								Call("then", js.FuncOf(func(this js.Value, args []js.Value) any {
									handleArrayBuffer(args[0])
									return nil
								})).
								Call("catch", js.FuncOf(func(this js.Value, args []js.Value) any {
									slog.Error("error trying to get array buffer from webmessage binary blob")
									return nil
								}))
						} else if data.InstanceOf(js.Global().Get("ArrayBuffer")) {
							handleArrayBuffer(data)
						} else {
							slog.Error("unrecognized type of binary message")
						}
					}
				}

				return nil
			}))

			wg.Wait()

			if result.closed {
				slog.Debug("intentionally marked as closed, aborting")
				return
			} else if reconnectStrategy == nil {
				slog.Debug("no reconnect strategy, aborting")
				return
			} else {
				retry, delayBeforeRetrying := reconnectStrategy(failuresInARow, lastFailureDelay)
				slog.Debug("recconnect", "retry", retry, "delay", delayBeforeRetrying)
				if retry {
					time.Sleep(delayBeforeRetrying)
				} else {
					return
				}
			}
		}
	}()

	return result
}

func (c *Connection) Close() {
	c.closed = true
	if c.value.Truthy() {
		c.value.Call("close")
	}
	c.wg.Wait()
}

var ErrWebsocketNotOpen = errors.New("websocket not open")

func (c *Connection) SendTextMessage(value string) error {
	if !c.value.Truthy() {
		return ErrWebsocketNotOpen
	}

	c.value.Call("send", value)

	return nil
}

func (c *Connection) SendBinaryMessage(value []byte) error {
	if !c.value.Truthy() {
		return ErrWebsocketNotOpen
	}

	buffer := js.Global().Get("Uint8Array").New(len(value))
	js.CopyBytesToJS(buffer, value)
	c.value.Call("send", buffer)

	return nil
}

func ConstantDelay(delay time.Duration) ReconnectStrategy {
	return func(failuresInARow int, lastFailureDelay time.Duration) (retry bool, delayBeforeRetrying time.Duration) {
		if failuresInARow == 0 {
			return true, 0
		}
		return true, delay
	}
}

func ExponentialBackoff(initialDelay, maxDelay time.Duration) ReconnectStrategy {
	return func(failuresInARow int, lastFailureDelay time.Duration) (retry bool, delayBeforeRetrying time.Duration) {
		if failuresInARow == 0 {
			return true, initialDelay
		}
		return true, min(lastFailureDelay*2, maxDelay)
	}
}
