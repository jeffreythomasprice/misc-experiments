package main

import (
	"log/slog"
	"shared"

	. "client/dom"
)

func main() {
	shared.InitSlog()

	var username, password string

	usernameChanged := func(e Event) {
		username = e.This().Get("value").String()
	}

	passwordChanged := func(e Event) {
		password = e.This().Get("value").String()
	}

	if err := Div(
		H1(Text("Hello, World!")),
		Form(
			Div(
				Label(
					Attr("for", "username"),
					Text("Username:"),
				),
				Input(
					Attr("name", "username"),
					Attr("placeholder", "Username"),
					Attr("type", "text"),
					EventHandler("change", usernameChanged),
					EventHandler("keyup", usernameChanged),
				),
			),
			Div(
				Label(
					Attr("for", "password"),
					Text("Password:"),
				),
				Input(
					Attr("name", "password"),
					Attr("placeholder", "Password"),
					Attr("type", "password"),
					EventHandler("change", passwordChanged),
					EventHandler("keyup", passwordChanged),
				),
			),
			EventHandler("submit", func(e Event) {
				e.PreventDefault()
				slog.Debug("TODO submit", "username", username, "password", password)
			}),
			Button(
				Attr("type", "submit"),
				Text("Log In"),
			),
		),
	).
		Swap("body", ReplaceChildren); err != nil {
		panic(err)
	}

	select {}
}
