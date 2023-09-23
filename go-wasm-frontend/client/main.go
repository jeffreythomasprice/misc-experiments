package main

import (
	"client/dom"
	"html/template"
	"log/slog"
	"net/http"
	"shared"
	"strings"
)

func main() {
	shared.InitSlog()

	go liveReload("ws://localhost:8000/_liveReload")

	loginPage()

	select {}
}

func loginPage() {
	document := dom.NewDocument()
	body := document.Body()

	replaceContent(body.Element, renderDomString(`
		<form id="form">
			<label>Enter a name:</label>
			<input type="text" name="name" placeholder="Name"/>
		</form>
	`)...)

	// TODO simpler casting?
	form := dom.NewHTMLFormElement(document.QuerySelector("#form").Value)

	// TODO event for when input becomes visible, set focus because autofocus doesn't work when swapping in

	handleFormSubmit(form, func(data *dom.FormData) {
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
	document := dom.NewDocument()
	body := document.Body()

	replaceContent(body.Element, loadingMessage()...)

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
	document := dom.NewDocument()
	body := document.Body()

	replaceContent(body.Element, renderDomString(`
		<form id="form">
			<label>Message:</label>
			<input type="text" name="message" placeholder="Message"/>
		</form>
	`)...)

	// TODO simpler casting?
	form := dom.NewHTMLFormElement(document.QuerySelector("#form").Value)
	messageInput := dom.NewHTMLInputElement(document.QuerySelector("#form > input[name='message']").Value)

	// TODO event for when input becomes visible, set focus because autofocus doesn't work when swapping in

	handleFormSubmit(form, func(data *dom.FormData) {
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

func loadingMessage() []*dom.Node {
	return renderDomString(`
		<div>Loading...</div>
	`)
}

func handleFormSubmit(form *dom.HTMLFormElement, f func(data *dom.FormData)) {
	isActive := false
	form.OnSubmit(func(e *dom.SubmitEvent) {
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

func replaceContent(target *dom.Element, newContent ...*dom.Node) {
	for _, child := range target.Children() {
		target.RemoveChild(child)
	}
	appendContent(target, newContent...)
}

func appendContent(target *dom.Element, newContent ...*dom.Node) {
	for _, child := range newContent {
		target.AppendChild(child)
	}
}

func renderDomString(s string) []*dom.Node {
	temp := dom.NewDocument().CreateElement("div")
	temp.SetInnerHTML(s)
	return temp.Children()
}

func newDomRenderer(f func() (string, error)) func() []*dom.Node {
	return func() []*dom.Node {
		s, err := f()
		if err != nil {
			// TODO handle error when rendering
			slog.Error("error rendering dom elements from string", "err", err)
			return nil
		}
		return renderDomString(s)
	}
}

func newTemplateRenderer(t *template.Template, name string, data func() any) func() []*dom.Node {
	return newDomRenderer(func() (string, error) {
		var s strings.Builder
		if err := t.ExecuteTemplate(&s, name, data()); err != nil {
			return "", nil
		}
		return s.String(), nil
	})
}

func newReplaceChildWith(target *dom.Element, f func() []*dom.Node) func() {
	return func() {
		replaceContent(target, f()...)
	}
}
