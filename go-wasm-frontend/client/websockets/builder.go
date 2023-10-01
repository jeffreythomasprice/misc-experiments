package websockets

import (
	"context"
	"log/slog"
	"syscall/js"
	"time"
)

type Builder struct {
	url       string
	protocols []string
	reconnect ReconnectStrategy
}

func NewBuilder(url string) *Builder {
	return &Builder{
		url: url,
	}
}

func (builder *Builder) Protocols(protocols []string) *Builder {
	builder.protocols = protocols
	return builder
}

func (builder *Builder) Reconnect(f ReconnectStrategy) *Builder {
	builder.reconnect = f
	return builder
}

func (builder *Builder) Build(ctx context.Context) (outgoing chan<- Event, incoming <-chan Event) {
	url := builder.url
	protocols := builder.protocols
	reconnect := builder.reconnect

	resultOutgoing := make(chan Event)
	resultIncoming := make(chan Event)
	outgoing = resultOutgoing
	incoming = resultIncoming

	go func() {
		defer close(resultOutgoing)
		defer close(resultIncoming)

		for {
			wsCtx, signalClosed := context.WithCancel(ctx)

			var ws js.Value
			if protocols != nil {
				ws = js.Global().Get("WebSocket").New(url, protocols)
			} else {
				ws = js.Global().Get("WebSocket").New(url)
			}

			ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket open", "url", url)
				if reconnect != nil {
					builder.reconnect.Reset()
				}

				go func() {
					for {
						select {
						// exit if ws closed
						case <-wsCtx.Done():
							return
						case e := <-resultOutgoing:
							if e.IsTextMessage() || e.IsBinaryMessage() {
								ws.Call("send", e.value)
							}
						}
					}
				}()

				resultIncoming <- Event{t: EventOpen}
				return nil
			}))

			ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket close", "url", url)
				resultIncoming <- Event{t: EventClose}
				signalClosed()
				return nil
			}))

			ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
				slog.Debug("websocket error", "url", url)
				select {
				case resultIncoming <- Event{t: EventError}:
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
					resultIncoming <- Event{t: EventTextMessage, value: data}
				} else {
					resultIncoming <- Event{t: EventBinaryMessage, value: data}
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
			reconnectDelay, shouldReconnect := reconnect.Next()
			slog.Debug("websocket reconnect", "url", url, "should reconnect", shouldReconnect, "reconnect delay", reconnectDelay)
			if !shouldReconnect {
				return
			}
			select {
			// exit if signalled
			case <-ctx.Done():
				return
			// otherwise wait for the desired delay
			case <-time.After(reconnectDelay):
			}
		}
	}()

	return
}
