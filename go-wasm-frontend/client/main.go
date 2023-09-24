package main

import (
	"client/dom"
	"log/slog"
	"net/http"
	"shared"
)

func main() {
	shared.InitSlog()

	go liveReload("ws://localhost:8000/_liveReload")

	loginPage()

	select {}
}

func loginPage() {
	document := dom.GetDocument()
	body := document.Body()

	replaceContent(body, renderDomString(`
		<form id="form">
			<label>Enter a name:</label>
			<input type="text" name="name" placeholder="Name"/>
		</form>
	`)...)

	form := dom.AsHTMLFormElement(document.QuerySelector("#form"))

	// TODO event for when input becomes visible, set focus because autofocus doesn't work when swapping in

	handleFormSubmit(form, func(data dom.FormData) {
		request := &shared.WebsocketLoginRequest{
			Name: data.Entries()["name"][0].String(),
		}
		result, err := shared.MakeJSONRequest[shared.WebsocketLoginResponse](http.MethodPost, "/login", request)
		if err != nil {
			slog.Error("error making login request", "err", err)
		} else {
			startWebsocket(result)
		}
	})
}

func startWebsocket(client *shared.WebsocketLoginResponse) {
	document := dom.GetDocument()
	body := document.Body()

	replaceContent(body, loadingMessage()...)

	ws := NewWebsocketWithReconnect("ws://localhost:8000/ws", nil, func(message string) {
		// TODO handle response messages
		slog.Debug("TODO handle response message", "message", message)
	})

	if err := ws.SendJSON(&shared.WebsocketClientToServerMessage{
		Type: shared.WebsocketClientToServerMessageTypeLogin,
		Login: &shared.WebsocketClientToServerMessageLogin{
			ID: client.ID,
		},
	}); err != nil {
		slog.Error("error sending websocket message", "err", err)
		return
	}

	loggedInPage(ws)
}

func loggedInPage(ws *WebsocketClient) {
	document := dom.GetDocument()
	body := document.Body()

	replaceContent(body, renderDomString(`
		<form id="form">
			<label>Message:</label>
			<input type="text" name="message" placeholder="Message"/>
		</form>
	`)...)

	form := dom.AsHTMLFormElement(document.QuerySelector("#form"))
	messageInput := dom.AsHTMLInputElement(document.QuerySelector("#form > input[name='message']"))

	// TODO event for when input becomes visible, set focus because autofocus doesn't work when swapping in

	handleFormSubmit(form, func(data dom.FormData) {
		message := data.Entries()["message"][0].String()
		messageInput.SetValue("")
		messageInput.Focus()
		if err := ws.SendJSON(&shared.WebsocketClientToServerMessage{
			Type: shared.WebsocketClientToServerMessageTypeSend,
			Send: &shared.WebsocketClientToServerMessageSend{
				Message: message,
			},
		}); err != nil {
			slog.Error("error sending websocket message", "err", err)
			// TODO handle error
		}
	})
}

func loadingMessage() []dom.Node {
	return renderDomString(`
		<div>Loading...</div>
	`)
}

func handleFormSubmit(form dom.HTMLFormElement, f func(data dom.FormData)) {
	isActive := false
	form.OnSubmit(func(e dom.SubmitEvent) {
		e.PreventDefault()
		if isActive {
			return
		}
		isActive = true
		data := e.FormData()
		go func() {
			f(data)
			isActive = false
		}()
	})
}

func replaceContent(target dom.Element, newContent ...dom.Node) {
	for _, child := range target.Children() {
		target.RemoveChild(child)
	}
	appendContent(target, newContent...)
}

func appendContent(target dom.Element, newContent ...dom.Node) {
	for _, child := range newContent {
		target.AppendChild(child)
	}
}

func renderDomString(s string) []dom.Node {
	temp := dom.GetDocument().CreateElement("div")
	temp.SetInnerHTML(s)
	return temp.Children()
}
