package websockets

import "syscall/js"

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
	panic("TODO implement binary messages")
}
