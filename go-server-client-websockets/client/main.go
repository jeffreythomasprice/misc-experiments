package main

import (
	"client/dom"
	"encoding/json"
	"fmt"
	"log/slog"
	"os"
	"shared/demo"
	"shared/websockets"
	"shared/websockets/reload"
	"strings"
	"syscall/js"

	"github.com/google/uuid"
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

	connection, err := websockets.NewWebsocketConnection(fmt.Sprintf("ws://%v/ws", host))
	if err != nil {
		// TODO error handling
		panic(err)
	}

	// TODO JEFF json websocket connection needs a wrapper for going from json.RawMessage to tagged union, same on server
	jsonConnection := websockets.NewJsonWebsocketConnection[json.RawMessage](connection)
	go func() {
		for rawMessage := range jsonConnection.Incoming() {
			message, err := demo.MessageTaggedUnion.Unmarshall(rawMessage)
			if err != nil {
				slog.Error("error unmarshalling message", "err", err)
				continue
			}
			switch t := message.(type) {
			case *demo.ServerInformClientsAboutMessage:
				panic("TODO JEFF handle this message type")
			case *demo.ServerInformClientsAboutNameChange:
				panic("TODO JEFF handle this message type")
			default:
				slog.Debug("unhandled message type", "type", t)
			}
		}
	}()

	clientName, err := uuid.NewRandom()
	if err != nil {

		// TODO error handling
		panic(err)
	}
	// TODO JEFF needs a helper for sending tagged unions
	if rawMessage, err := demo.MessageTaggedUnion.Marshal(&demo.ClientSetName{
		Name: clientName.String(),
	}); err != nil {
		// TODO error handling
		panic(err)
	} else if err := jsonConnection.Send(rawMessage); err != nil {
		// TODO error handling
		panic(err)
	}

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
