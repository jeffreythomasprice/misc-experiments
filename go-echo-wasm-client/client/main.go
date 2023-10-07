package main

import (
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"shared"

	. "client/dom"
)

func main() {
	shared.InitSlog()

	// TODO router, parse window.location and draw some components replacing a given selector

	response, err := shared.CheckToken()
	if err != nil {
		var statusCodeErr *shared.HTTPResponseError
		if errors.As(err, &statusCodeErr) {
			loginPage(defaultLoginContent)
		} else {
			errorPage(err.Error())
		}
	} else {
		defaultLoginContent(response)
	}

	select {}
}

func defaultLoginContent(response *shared.LoginResponse) {
	loggedInPage(
		response,
		Div(Text("TODO some real logged in content")),
	)
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
						var httpErr *shared.HTTPResponseError
						if errors.As(err, &httpErr) {
							if httpErr.Response.StatusCode == http.StatusUnauthorized {
								errorMessage("Invalid credentials")
								return
							}
						}
						errorPage(fmt.Sprintf("Login failed: %v", err))
						return
					}
					success(response)
				}()
			}),
			Button(
				Attr("type", "submit"),
				Text("Log In"),
			),
		),
		Div(Attr("id", "errorMessages")),
	).
		Swap("body", ReplaceChildren); err != nil {
		panic(err)
	}
}

func loggedInPage(user *shared.LoginResponse, content ...Renderer) {
	claims, err := shared.ParseJWTClaimsUnverified(user.Token)
	if err != nil {
		slog.Error("failed to parse jwt", "err", err)
		errorPage("Failed to parse login token")
		return
	}

	Div(append(
		[]Renderer{Div(
			Div(Textf("Logged in as: %v", claims.Username)),
			Button(
				Text("Log Out"),
				EventHandler("click", func(e Event) {
					go func() {
						if err := shared.Logout(); err != nil {
							errorPage(fmt.Sprintf("Logout failed: %v", err))
						} else {
							loginPage(defaultLoginContent)
						}
					}()
				}),
			),
		)},
		content...,
	)...).
		Swap("body", ReplaceChildren)
}

func errorMessage(msg string) {
	errorContent(msg).Swap("#errorMessages", Append)
}

func errorPage(msg string) {
	errorContent(msg).Swap("body", ReplaceChildren)
}

func errorContent(msg string) *Element {
	return Div(
		P(
			Class("error"),
			Textf("Error: %v", msg),
		),
	)
}
