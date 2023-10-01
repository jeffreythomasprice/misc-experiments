package websockets

import (
	"context"
	"syscall/js"
)

type EventType = int

const (
	EventOpen EventType = iota
	EventClose
	EventError
	EventTextMessage
	EventBinaryMessage
)

type Event struct {
	t     EventType
	value js.Value
}

func NewTextMessage(value string) Event {
	return Event{
		t:     EventTextMessage,
		value: js.ValueOf(value),
	}
}

func NewBinaryMessage(value []byte) Event {
	jsArray := js.Global().Get("Uint8Array").New(len(value))
	_ = js.CopyBytesToJS(jsArray, value)
	return Event{
		t:     EventBinaryMessage,
		value: jsArray,
	}
}

func (e Event) Type() EventType {
	return e.t
}

func (e Event) IsTextMessage() bool {
	return e.Type() == EventTextMessage
}

func (e Event) Text() string {
	if !e.IsTextMessage() {
		panic("not a text message")
	}
	return e.value.String()
}

func (e Event) IsBinaryMessage() bool {
	return e.Type() == EventBinaryMessage
}

func (e Event) Binary() []byte {
	if !e.IsBinaryMessage() {
		panic("not a binary message")
	}
	if e.value.InstanceOf(js.Global().Get("ArrayBuffer")) {
		return arrayBufferToBytes(e.value)
	} else if e.value.InstanceOf(js.Global().Get("Blob")) {
		arrayBuffer, err := await(e.value.Call("arrayBuffer"))
		if err.Truthy() {
			panic(err.String())
		}
		return arrayBufferToBytes(arrayBuffer)
	} else {
		panic("unhandled binary message type")
	}
}

func await(promise js.Value) (result js.Value, err js.Value) {
	ctx, done := context.WithCancel(context.Background())
	promise.
		Call("then", js.FuncOf(func(this js.Value, args []js.Value) any {
			result = args[0]
			done()
			return nil
		})).
		Call("catch", js.FuncOf(func(this js.Value, args []js.Value) any {
			err = args[0]
			done()
			return nil
		}))
	<-ctx.Done()
	return
}

func arrayBufferToBytes(arrayBuffer js.Value) []byte {
	jsArray := js.Global().Get("Uint8Array").New(arrayBuffer)
	result := make([]byte, jsArray.Length())
	_ = js.CopyBytesToGo(result, jsArray)
	return result
}
