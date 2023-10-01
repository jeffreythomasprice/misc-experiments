package websockets

import "syscall/js"

type WebsocketEventType = int

const (
	WebsocketEventOpen WebsocketEventType = iota
	WebsocketEventClose
	WebsocketEventError
	WebsocketEventTextMessage
	WebsocketEventBinaryMessage
)

type WebsocketEvent struct {
	t     WebsocketEventType
	value js.Value
}

func NewWebsocketTextMessage(value string) WebsocketEvent {
	return WebsocketEvent{
		t:     WebsocketEventTextMessage,
		value: js.ValueOf(value),
	}
}

func NewWebsocketBinaryMessage(value []byte) WebsocketEvent {
	panic("TODO imlement binary message")
}

func (e WebsocketEvent) Type() WebsocketEventType {
	return e.t
}

func (e WebsocketEvent) IsTextMessage() bool {
	return e.Type() == WebsocketEventTextMessage
}

func (e WebsocketEvent) Text() string {
	if !e.IsTextMessage() {
		panic("not a text message")
	}
	return e.value.String()
}

func (e WebsocketEvent) IsBinaryMessage() bool {
	return e.Type() == WebsocketEventBinaryMessage
}

func (e WebsocketEvent) Binary() []byte {
	if !e.IsBinaryMessage() {
		panic("not a binary message")
	}
	panic("TODO implement binary messages")
}
