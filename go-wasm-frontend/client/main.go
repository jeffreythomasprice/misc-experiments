package main

import (
	"client/dom"
	"fmt"
	"log/slog"
	"net/http"
	"shared"
	"strings"
)

func main() {
	shared.InitSlog()

	go liveReload("ws://localhost:8000/_liveReload")

	appendCss(`
		.error {
			color: red;
			font-weight: bold;
		}
	`)

	loginPage()

	select {}
}

func loginPage() {
	showPage(renderDomString(`
		<form id="form">
			<label>Enter a name:</label>
			<input type="text" name="name" placeholder="Name"/>
		</form>
	`)...)

	document := dom.GetDocument()
	form := dom.AsHTMLFormElement(document.QuerySelector("#form"))
	nameInput := dom.AsHTMLInputElement(document.QuerySelector("#form > input[name='name']"))

	waitForElementToAppear(
		func(e dom.Element) {
			nameInput.Focus()
		},
		nameInput,
	)

	handleFormSubmit(form, func(data dom.FormData) {
		request := &shared.WebsocketLoginRequest{
			Name: data.Entries()["name"][0].String(),
		}
		result, err := shared.MakeJSONRequest[shared.WebsocketLoginResponse](http.MethodPost, "/login", request)
		if err != nil {
			slog.Error("error making login request", "err", err)
		} else {
			startWebsocket(request.Name, result)
		}
	})
}

func startWebsocket(name string, client *shared.WebsocketLoginResponse) {
	showPage(loadingMessage()...)

	ws := NewWebsocketWithReconnect("ws://localhost:8000/ws", nil, func(messageStr string) {
		message, err := shared.UnmarshalJson[shared.WebsocketServerToClientMessage](strings.NewReader(messageStr))
		if err != nil {
			showErrorMessage(fmt.Sprintf("Error parsing incoming message from server:\n%v", err))
			return
		}

		switch message.Type {
		case shared.WebsocketServerToClientMessageTypeSend:
			appendContent(
				dom.GetDocument().QuerySelector("#messages"),
				renderDomStringf(
					"<div>%v - %v</div>",
					message.Send.SenderID,
					message.Send.Message,
				)...,
			)

		default:
			showErrorMessage(fmt.Sprintf("Unhandled incoming message type from server: %v", message.Type))
		}
	})

	if err := ws.SendJSON(&shared.WebsocketClientToServerMessage{
		Type: shared.WebsocketClientToServerMessageTypeLogin,
		Login: &shared.WebsocketClientToServerMessageLogin{
			ID: client.ID,
		},
	}); err != nil {
		showErrorMessage(fmt.Sprintf("Error sending websocket message:\n%v", err))
		return
	}

	loggedInPage(name, client.ID, ws)
}

func loggedInPage(name, id string, ws *WebsocketClient) {
	showPage(renderDomStringf(
		`
		<div>Name: %v</div>
		<div>ID: %v</div>
		<form id="form">
			<label>Message:</label>
			<input type="text" name="message" placeholder="Message"/>
		</form>
		<div id="messages"></div>
		`,
		name,
		id,
	)...)

	document := dom.GetDocument()
	form := dom.AsHTMLFormElement(document.QuerySelector("#form"))
	messageInput := dom.AsHTMLInputElement(document.QuerySelector("#form > input[name='message']"))

	waitForElementToAppear(
		func(e dom.Element) {
			messageInput.Focus()
		},
		messageInput,
	)

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
			showErrorMessage(fmt.Sprintf("Error sending websocket message:\n%v", err))
		}
	})
}

func loadingMessage() []dom.Node {
	return renderDomString(`
		<div>Loading...</div>
	`)
}

func showErrorMessage(message string) {
	appendContent(
		dom.GetDocument().Body(),
		renderDomStringf(
			`<p class="error">%v</p>`,
			message,
		)...,
	)
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

func waitForElementToAppear(callback func(e dom.Element), e dom.Element) {
	obs := dom.NewIntersectionObserver(func(entries []dom.IntersectionObserverEntry, observer dom.IntersectionObserver) {
		for _, e := range entries {
			callback(e.Target())
			observer.Unobserve(e.Target())
		}
		observer.Disconnect()
	}, nil)
	obs.Observe(e)
}

func showPage(newContent ...dom.Node) {
	replaceContent(
		dom.GetDocument().Body(),
		newContent...,
	)
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

func renderDomStringf(s string, args ...any) []dom.Node {
	return renderDomString(fmt.Sprintf(s, args...))
}

func appendCss(css string) {
	style := dom.GetDocument().CreateElement("style")
	style.SetAttribute("type", "text/css")
	style.SetInnerHTML(css)
	dom.GetDocument().Head().AppendChild(style)
}
