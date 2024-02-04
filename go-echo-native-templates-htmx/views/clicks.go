package views

import (
	"context"
	_ "embed"
	"io"
)

//go:embed clicks.html
var clicksSource string
var clicksPageFunc templateFunc = page(clicksSource, "index")
var clicksResponseFunc templateFunc = snippet(clicksSource, "response")

func ClicksPage(ctx context.Context, w io.Writer, clicks uint64) error {
	return clicksPageFunc(ctx, w, struct{ Clicks uint64 }{clicks})
}

func ClicksResponse(ctx context.Context, w io.Writer, clicks uint64) error {
	return clicksResponseFunc(ctx, w, struct{ Clicks uint64 }{clicks})
}
