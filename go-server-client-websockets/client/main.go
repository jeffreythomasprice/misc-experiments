package main

import (
	"client/dom"
	"fmt"
	"log/slog"
	"os"
	"shared"
	"shared/websockets"
	"shared/websockets/reload"
	"strings"
	"syscall/js"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level:     slog.LevelDebug,
			AddSource: false,
		},
	)))

	host := js.Global().Get("window").Get("location").Get("host")
	reload.StartAutoReloadClient(fmt.Sprintf("ws://%v/ws/autoreload", host))

	// TODO JEFF demo
	connection, err := websockets.NewWebsocketConnection(fmt.Sprintf("ws://%v/ws", host))
	if err != nil {
		panic(err)
	}
	jsonConnection := websockets.NewJsonWebsocketConnection[shared.Message](connection)
	go func() {
		for message := range jsonConnection.Incoming() {
			slog.Info("incoming message", "text", message.Message)
		}
	}()
	jsonConnection.Send(shared.Message{
		Message: "Hello from client",
	})

	dom.MustGetDomElementByQuerySelector("body").
		ReplaceChildren(
			dom.MustNewDomElementFromHtmlString(`
				<input id="messageInput" type="text"></input>
				<button id="submitMessageButton" type="button">Submit</button>
			`),
		)

	messageInput := dom.MustGetDomElementById("messageInput")
	submitMessageButton := dom.MustGetDomElementById("submitMessageButton")

	submit := func() {
		value := messageInput.Get("value").String()
		messageInput.Set("value", "")
		messageInput.Call("focus")

		if len(value) > 0 {
			slog.Debug("TODO JEFF submitting", "value", value)
		}
	}

	messageInput.Set("onkeypress", js.FuncOf(func(this js.Value, args []js.Value) any {
		event := args[0]
		key := strings.ToLower(event.Get("key").String())
		if key == "enter" {
			submit()
		}
		return nil
	}))

	submitMessageButton.Set("onclick", js.FuncOf(func(this js.Value, args []js.Value) any {
		submit()
		return nil
	}))

	messageInput.Call("focus")

	select {}
}
