package websockets

import (
	"context"
	"log/slog"
	"syscall/js"
	"time"
)

type websocketClientReconnectStrategy func() (time.Duration, bool)

type websocketClientBuilder struct {
	url       string
	protocols []string
	reconnect websocketClientReconnectStrategy
}

func NewWebsocketBuilder(url string) *websocketClientBuilder {
	return &websocketClientBuilder{
		url: url,
	}
}

func (builder *websocketClientBuilder) Protocols(protocols []string) *websocketClientBuilder {
	builder.protocols = protocols
	return builder
}

func (builder *websocketClientBuilder) Reconnect(f websocketClientReconnectStrategy) *websocketClientBuilder {
	builder.reconnect = f
	return builder
}

func (builder *websocketClientBuilder) Build(ctx context.Context) (outgoing chan<- WebsocketEvent, incoming <-chan WebsocketEvent) {
	url := builder.url
	protocols := builder.protocols
	reconnect := builder.reconnect

	resultOutgoing := make(chan WebsocketEvent)
	resultIncoming := make(chan WebsocketEvent)
	outgoing = resultOutgoing
	incoming = resultIncoming

	var ws js.Value
	go func() {
		for e := range resultOutgoing {
			currentWs := ws
			if currentWs.Truthy() && (e.IsTextMessage() || e.IsBinaryMessage()) {
				currentWs.Call("send", e.value)
			}
		}
	}()

	go func() {
		defer close(resultOutgoing)
		defer close(resultIncoming)

		for {
			wsCtx, signalClosed := context.WithCancel(ctx)

			if protocols != nil {
				ws = js.Global().Get("WebSocket").New(url, protocols)
			} else {
				ws = js.Global().Get("WebSocket").New(url)
			}

			ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket open", "url", url)
				resultIncoming <- WebsocketEvent{t: WebsocketEventOpen}
				return nil
			}))

			ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket close", "url", url)
				resultIncoming <- WebsocketEvent{t: WebsocketEventClose}
				signalClosed()
				return nil
			}))

			ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket error", "url", url)
				select {
				case resultIncoming <- WebsocketEvent{t: WebsocketEventError}:
				case <-ctx.Done():
				case <-wsCtx.Done():
				}
				ws.Call("close")
				return nil
			}))

			ws.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket error", "message", url)
				data := args[0].Get("data")
				if data.Type() == js.TypeString {
					resultIncoming <- WebsocketEvent{t: WebsocketEventTextMessage, value: data}
				} else {
					resultIncoming <- WebsocketEvent{t: WebsocketEventBinaryMessage, value: data}
				}
				return nil
			}))

			select {
			// exit if signalled
			case <-ctx.Done():
				// clean up if exiting early
				signalClosed()
				return
			// websocket is closed
			case <-wsCtx.Done():
				slog.Debug("websocket disconnected", "url", url)
			}

			if reconnect == nil {
				slog.Debug("websocket has no reconnect strategy", "url", url)
				return
			}
			reconnectDelay, shouldReconnect := reconnect()
			slog.Debug("websocket reconnect", "url", url, "should reconnect", shouldReconnect, "reconnect delay", reconnectDelay)
			if !shouldReconnect {
				return
			}
			// TODO support early exit during sleep
			time.Sleep(reconnectDelay)

			select {
			// exit if signalled
			case <-ctx.Done():
				return
			default:
			}
		}
	}()

	return
}
