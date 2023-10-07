package main

import (
	"errors"
	"log/slog"
	"shared"
	"time"

	. "client/dom"
)

func main() {
	shared.InitSlog()

	// TODO router, parse window.location and draw some components replacing a given selector

	response, err := shared.CheckToken()
	if err != nil {
		var statusCodeErr *shared.HTTPResponseError
		if errors.As(err, &statusCodeErr) {
			loginPage(func(lr *shared.LoginResponse) {
				loggedInPage(lr)
			})
		} else {
			errorPage(err.Error())
		}
	} else {
		loggedInPage(response)
	}

	select {}
}

func loginPage(success func(*shared.LoginResponse)) {
	var username, password string

	usernameChanged := func(e Event) {
		username = e.This().Get("value").String()
	}

	passwordChanged := func(e Event) {
		password = e.This().Get("value").String()
	}

	if err := Div(
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
				go func() {
					response, err := shared.Login(&shared.LoginRequest{
						Username: username,
						Password: password,
					})
					if err != nil {
						slog.Error("error logging in", "err", err)
					} else {
						success(response)
					}
				}()
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
}

func loggedInPage(user *shared.LoginResponse) {
	claims, err := shared.ParseJWTClaimsUnverified(user.Token)
	if err != nil {
		slog.Error("failed to parse jwt", "err", err)
		errorPage("Failed to parse login token")
		return
	}
	// TODO some real content
	Div(
		P(Text("Logged in page")),
		P(Textf("username = %v", claims.Username)),
		P(Textf("expires = %v", time.Unix(claims.ExpiresAt, 0).Format(time.RFC3339))),
	).
		Swap("body", ReplaceChildren)
}

func errorPage(msg string) {
	Div(
		P(Textf("Error: %v", msg)),
	).
		Swap("body", ReplaceChildren)
}
