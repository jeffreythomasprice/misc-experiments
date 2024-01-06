package utils

import (
	"context"
	"net/http"

	"github.com/a-h/templ"
	"github.com/rs/zerolog/log"
)

func RenderToHttpResponse(c templ.Component, w http.ResponseWriter) {
	if err := c.Render(context.Background(), w); err != nil {
		// TODO response with non-200
		log.Err(err).Msg("failed to render http response")
	}
}
