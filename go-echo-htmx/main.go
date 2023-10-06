package main

import (
	"html/template"
	"io"
	"log/slog"
	"net/http"
	"os"
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level:     slog.LevelDebug,
		AddSource: true,
	})))

	e := echo.New()
	e.HideBanner = true
	e.HidePort = true
	e.Debug = true

	t := &appTemplates{
		templates: template.Must(template.New("index").Parse(`
			<!DOCTYPE html>
			<html lang="en">
				<head>
					<meta charset="utf-8">
					<script src="https://unpkg.com/htmx.org@1.9.6"></script>
					<script>
						htmx.logAll();
					</script>
				</head>
				<body>
					{{.content}}
				</body>
			</html>
			{{define "content"}}
			<h1>Hello, World!</h1>
			{{end}}
		`)),
	}
	e.Renderer = t

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	e.GET("/", func(c echo.Context) error {
		var s strings.Builder
		if err := t.Render(&s, "content", nil, c); err != nil {
			return err
		}
		return c.Render(http.StatusOK, "index", map[string]any{"content": template.HTML(s.String())})
	})
	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

type appTemplates struct {
	templates *template.Template
}

var _ echo.Renderer = (*appTemplates)(nil)

// Render implements echo.Renderer.
func (t *appTemplates) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.templates.ExecuteTemplate(w, name, data)
}
