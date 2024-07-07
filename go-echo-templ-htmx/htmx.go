package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog/log"
)

func htmxRedirect(c echo.Context, url string) error {
	log.Trace().Str("url", url).Msg("redirecting")
	if isHtmxRequest(c) {
		c.Response().Header().Set("hx-location", url)
		return nil
	} else {
		return c.Redirect(http.StatusFound, url)
	}
}

func isHtmxRequest(c echo.Context) bool {
	value := c.Request().Header.Get("hx-request")
	return len(value) > 0
}
