package views

import (
	"context"
	_ "embed"
	"io"
)

//go:embed login.html
var loginSource string
var notLoggedInPageFunc templateFunc = page(loginSource, "form")
var loggedInPageFunc templateFunc = page(loginSource, "success")
var loggedInResponseSnippetFunc templateFunc = snippet(loginSource, "success")

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
