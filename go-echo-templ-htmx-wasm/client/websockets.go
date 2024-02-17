package main

import (
	"encoding/json"
	"fmt"
	"strings"
	"syscall/js"
	"time"

	"github.com/rs/zerolog/log"
)

func WebsocketUrl(path string) string {
	loc := js.Global().Get("window").Get("location")

	protocol := loc.Get("protocol").String()
	switch protocol {
	case "http:":
		protocol = "ws:"
	case "https:":
		protocol = "wss:"
	default:
		log.Panic().
			Str("location", loc.String()).
			Str("protocol", protocol).
			Msg("failed to determine protocol")
	}

	host := strings.TrimSuffix(loc.Get("host").String(), "/")

	path = strings.TrimPrefix(path, "/")

	return fmt.Sprintf("%s//%s/%s", protocol, host, path)
}

func WebsocketClient[Send, Receive any](url string) (chan<- Send, <-chan Receive) {
	log := log.With().Str("websocket url", url).Logger()

	ws := js.Global().Get("WebSocket").New(url)

	send := make(chan Send)
	receive := make(chan Receive)

	openedOrClosed := make(chan struct{})
	signalOpenedOrClosed := func() {
		select {
		case openedOrClosed <- struct{}{}:
		default:
		}
	}

	ws.Call("addEventListener", "open", js.FuncOf(func(this js.Value, args []js.Value) any {
		log.Trace().Msg("opened")
		signalOpenedOrClosed()
		return nil
	}))

	ws.Call("addEventListener", "close", js.FuncOf(func(this js.Value, args []js.Value) any {
		log.Trace().Msg("closed")
		close(send)
		close(receive)
		defer close(openedOrClosed)
		signalOpenedOrClosed()
		return nil
	}))

	ws.Call("addEventListener", "error", js.FuncOf(func(this js.Value, args []js.Value) any {
		log.Trace().Msg("error")
		return nil
	}))

	ws.Call("addEventListener", "message", js.FuncOf(func(this js.Value, args []js.Value) any {
		log.Trace().Msg("message")

		msg := args[0].Get("data")
		if msg.Type() == js.TypeString {
			msgStr := msg.String()
			var msgObj Receive
			if err := json.Unmarshal([]byte(msgStr), &msgObj); err != nil {
				log.Error().Err(err).Msg("failed to deserialize incoming message")
				return nil
			}
			receive <- msgObj
		} else if msg.InstanceOf(js.Global().Get("ArrayBuffer")) {
			log.Panic().Msg("implement ArrayBuffer parsing")
		} else if msg.InstanceOf(js.Global().Get("Blob")) {
			log.Panic().Msg("implement Blob parsing")
		} else {
			log.Error().Str("type", msg.Type().String()).Msg("unhandled message type")
		}

		return nil
	}))

	go func() {
		for msg := range send {
			b, err := json.Marshal(msg)
			if err != nil {
				log.Error().Err(err).Msg("failed to serialize outgoing message")
				continue
			}
			buf := js.Global().Get("Uint8Array").New(len(b))
			js.CopyBytesToJS(buf, b)
			ws.Call("send", buf)
		}
	}()

	<-openedOrClosed

	return send, receive
}

func WebsocketClientStayConnectedForever[Send, Receive any](url string) (chan<- Send, <-chan Receive) {
	log := log.With().Str("websocket url", url).Logger()

	resultSend := make(chan Send)
	resultReceive := make(chan Receive)

	go func() {
		for {
			wsSend, wsReceive := WebsocketClient[Send, Receive](url)

			exitSignal := make(chan struct{})

			go func() {
				for {
					select {
					case <-exitSignal:
						close(exitSignal)
						return
					case msg := <-resultSend:
						wsSend <- msg
					}
				}
			}()

			for msg := range wsReceive {
				resultReceive <- msg
			}
			exitSignal <- struct{}{}

			// TODO increasing backoff
			sleepTime := time.Second * 2
			log.Trace().Str("sleepTime", sleepTime.String()).Msg("waiting before connecting again")
			time.Sleep(sleepTime)
		}
	}()

	return resultSend, resultReceive
}
