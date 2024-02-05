package views

import (
	"context"
	_ "embed"
	"io"
)

//go:embed login.html
var loginSource string
var notLoggedInPageFunc templateFunc = page(loginSource, "notLoggedIn")
var loggedInPageFunc templateFunc = page(loginSource, "loggedIn")
var loggedInResponseSnippetFunc templateFunc = snippet(loginSource, "response")

type User struct {
	Username string
	IsAdmin  bool
}

func NotLoggedInPage(ctx context.Context, w io.Writer) error {
	return notLoggedInPageFunc(ctx, w, nil)
}

func LoggedInPage(ctx context.Context, w io.Writer, data User) error {
	return loggedInPageFunc(ctx, w, data)
}

func LoggedInResponse(ctx context.Context, w io.Writer, data User) error {
	return loggedInResponseSnippetFunc(ctx, w, data)
}
