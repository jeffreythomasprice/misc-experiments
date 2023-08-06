//go:build js && wasm

package websockets

import (
	"log/slog"
	"shared"
	"sync"
	"syscall/js"
)

type websocketConnectionImpl struct {
	byteSlicePool sync.Pool

	incomingMutex       sync.Mutex
	incoming            chan []byte
	outgoingStringMutex sync.Mutex
	outgoingString      chan string
	outgoingBinaryMutex sync.Mutex
	outgoingBinary      chan []byte
}

var _ WebsocketConnection = (*websocketConnectionImpl)(nil)

func NewWebsocketConnection(addr string) (WebsocketConnection, error) {
	slog.Debug("connecting to websocket", "remote addr", addr)

	ws := js.Global().Get("WebSocket").New(addr)

	uint8ArrayConstructor := js.Global().Get("Uint8Array")

	result := &websocketConnectionImpl{
		byteSlicePool: sync.Pool{
			New: func() any {
				// just guess a random size, we'll resize them as needed later
				return make([]byte, 1024)
			},
		},

		incoming:       make(chan []byte),
		outgoingString: make(chan string),
		outgoingBinary: make(chan []byte),
	}

	ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Debug(
			"websocket connected",
			"remote addr", addr,
		)

		go func() {
			for message := range result.outgoingString {
				ws.Call("send", message)
			}
		}()

		go func() {
			for message := range result.outgoingBinary {
				uint8Array := uint8ArrayConstructor.New(len(message))
				js.CopyBytesToJS(uint8Array, message)
				ws.Call("send", uint8Array)
			}
		}()

		return nil
	}))

	ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Debug(
			"websocket disconnected",
			"remote addr", addr,
		)

		result.incomingMutex.Lock()
		defer result.incomingMutex.Unlock()
		close(result.incoming)
		result.incoming = nil

		result.outgoingStringMutex.Lock()
		defer result.outgoingStringMutex.Unlock()
		close(result.outgoingString)
		result.outgoingString = nil

		result.outgoingBinaryMutex.Lock()
		defer result.outgoingBinaryMutex.Unlock()
		close(result.outgoingBinary)
		result.outgoingBinary = nil

		return nil
	}))

	ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
		slog.Warn(
			"websocket error",
			"remote addr", addr,
		)
		return nil
	}))

	ws.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
		data := args[0].Get("data")
		if data.Type() == js.TypeString {
			dataBytes := []byte(data.String())
			slog.Debug(
				"received message",
				"remote addr", addr,
				"type", "string",
				"length", len(dataBytes),
			)
			result.incoming <- dataBytes
		} else if data.InstanceOf(js.Global().Get("Blob")) {
			data.Call("arrayBuffer").Call("then", js.FuncOf(func(this js.Value, args []js.Value) any {
				arrayBuffer := args[0]

				dataBytes := result.byteSlicePool.Get().([]byte)
				desiredLen := arrayBuffer.Get("byteLength").Int()
				dataBytes = shared.SetSliceLen(dataBytes, desiredLen)
				js.CopyBytesToGo(dataBytes, uint8ArrayConstructor.New(arrayBuffer))

				result.incomingMutex.Lock()
				defer result.incomingMutex.Unlock()
				result.incoming <- dataBytes

				return nil
			}))
		} else {
			slog.Warn("received message, but it wasn't a string or Blob")
		}
		return nil
	}))

	return result, nil
}

// Incoming implements WebsocketConnection.
func (connection *websocketConnectionImpl) Incoming() <-chan []byte {
	return connection.incoming
}

// ReturnIncoming implements WebsocketConnection.
func (connection *websocketConnectionImpl) ReturnIncoming(b []byte) {
	connection.byteSlicePool.Put(b)
}

// Send implements WebsocketConnection.
func (connection *websocketConnectionImpl) SendText(s string) error {
	connection.outgoingStringMutex.Lock()
	defer connection.outgoingStringMutex.Unlock()
	if connection.outgoingString == nil {
		return ErrClosed
	}
	connection.outgoingString <- s
	return nil
}

// Send implements WebsocketConnection.
func (connection *websocketConnectionImpl) SendBinary(b []byte) error {
	connection.outgoingBinaryMutex.Lock()
	defer connection.outgoingBinaryMutex.Unlock()
	if connection.outgoingBinary == nil {
		return ErrClosed
	}
	connection.outgoingBinary <- b
	return nil
}
