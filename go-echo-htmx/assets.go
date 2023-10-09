package main

import (
	"embed"
	"experiment/db"
	"html/template"
	"io"
	"net/http"
	"strings"

	"github.com/labstack/echo/v4"
)

//go:embed assets/*
var embedAssets embed.FS

type assetTemplates struct {
	*template.Template
}

var _ echo.Renderer = (*assetTemplates)(nil)

// Render implements echo.Renderer.
func (t *assetTemplates) Render(w io.Writer, name string, data any, c echo.Context) error {
	return t.ExecuteTemplate(w, name, data)
}

func (t *assetTemplates) RenderPage(c echo.Context, name string, data any) error {
	var s strings.Builder
	if err := t.ExecuteTemplate(&s, name, data); err != nil {
		return err
	}
	return c.Render(http.StatusOK, "index.html", template.HTML(s.String()))
}

func (t *assetTemplates) RenderLoginPage(c echo.Context) error {
	return t.RenderPage(c, "loginForm.html", nil)
}

func (t *assetTemplates) RenderLoggedInPage(c echo.Context, user *db.User) error {
	return t.RenderPage(c, "loggedInContent.html", user)
}

func (t *assetTemplates) RenderErrorMessage(c echo.Context, msg string) error {
	return c.Render(http.StatusOK, "errorMessage.html", msg)
}
