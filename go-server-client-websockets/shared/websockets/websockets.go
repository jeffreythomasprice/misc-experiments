package websockets

import (
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
)

type WebsocketServer interface {
	Close() error
	Incoming() <-chan WebsocketConnection
}

type WebsocketConnection interface {
	/*
		Incoming is the bytes received from the other end of the connection. Note that you should return these bytes to the connection for
		re-use, they might be coming from a pool.

			for x := range connection.Incoming() {
				// handle x
				connection.ReturnIncoming(x)
			}
	*/
	Incoming() <-chan []byte
	/*
		ReturnIncoming should be called on the results from Incoming, these might be coming from a pool and need to be put back to avoid
		excessive allocations.
	*/
	ReturnIncoming([]byte)
	SendText(string) error
	SendBinary([]byte) error
}

type JsonWebsocketConnection[T any] struct {
	connection WebsocketConnection
	incoming   chan T
}

var ErrClosed = errors.New("closed websocket connection")

func NewJsonWebsocketConnection[T any](connection WebsocketConnection) JsonWebsocketConnection[T] {
	incoming := make(chan T)
	go func() {
		for message := range connection.Incoming() {
			var typedMessage T
			if err := json.Unmarshal(message, &typedMessage); err != nil {
				slog.Warn("websocket json unmarshal error", "err", err)
			} else {
				incoming <- typedMessage
			}
			connection.ReturnIncoming(message)
		}
	}()
	return JsonWebsocketConnection[T]{
		connection,
		incoming,
	}
}

func (connection *JsonWebsocketConnection[T]) Incoming() <-chan T {
	return connection.incoming
}

func (connection *JsonWebsocketConnection[T]) Send(message T) error {
	b, err := json.Marshal(message)
	if err != nil {
		return fmt.Errorf("websocket json marshal error: %w", err)
	}
	return connection.connection.SendBinary(b)
}
