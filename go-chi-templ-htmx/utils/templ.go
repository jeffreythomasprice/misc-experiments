package utils

import (
	"context"
	"strings"

	"github.com/a-h/templ"
)

func RenderToString(c templ.Component) (string, error) {
	var result strings.Builder
	if err := c.Render(context.Background(), &result); err != nil {
		return "", err
	}
	return result.String(), nil
}
