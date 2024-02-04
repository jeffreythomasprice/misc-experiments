package views

import (
	"context"
	_ "embed"
	"io"
)

//go:embed login.html
var loginSource string
var loginFormFunc templateFunc = page(loginSource, "form")
var loginSuccessFunc templateFunc = snippet(loginSource, "success")

type User struct {
	Username string
}

func NotLoggedInPage(ctx context.Context, w io.Writer) error {
	return loginFormFunc(ctx, w, nil)
}

func LoggedInResponse(ctx context.Context, w io.Writer, data User) error {
	return loginSuccessFunc(ctx, w, data)
}
