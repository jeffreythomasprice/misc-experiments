package views

import (
	"context"
	_ "embed"
	"io"
)

//go:embed errors.html
var errorsSource string
var errorsResponseFunc templateFunc = snippet(errorsSource, "errors")

func ErrorsResponse(ctx context.Context, w io.Writer, messages ...string) error {
	return errorsResponseFunc(ctx, w, struct{ Messages []string }{messages})
}
