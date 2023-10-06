package main

import (
	"log/slog"
	"net/http"
	"os"
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/components"
	. "github.com/maragudk/gomponents/html"
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

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	e.GET("/", htmlNodeHandler(func() ([]Node, error) {
		return page(index)
	}))

	clicks := 0
	e.POST("/clicks", htmlNodeHandler(func() ([]Node, error) {
		clicks++
		return clickResults(clicks)
	}))

	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

func index() ([]Node, error) {
	return []Node{
		H1(Text("Hello, World!")),
		Button(
			Attr("hx-post", "/clicks"),
			Attr("hx-swap", "innerHTML"),
			Attr("hx-target", "#clickResults"),
			Text("Click Me"),
		),
		Div(ID("clickResults")),
	}, nil
}

func clickResults(clicks int) ([]Node, error) {
	return []Node{
		Div(Textf("Clicks: %d", clicks)),
	}, nil
}

func page(content func() ([]Node, error)) ([]Node, error) {
	c, err := content()
	if err != nil {
		return nil, err
	}
	return []Node{
		HTML5(HTML5Props{
			Head: []Node{
				Script(
					Src("https://unpkg.com/htmx.org@1.9.6"),
					Text("htmx.logAll();"),
				),
			},
			Body: c,
		}),
	}, nil
}

func htmlNodeHandler(f func() ([]Node, error)) echo.HandlerFunc {
	return func(c echo.Context) error {
		nodes, err := f()
		if err != nil {
			return err
		}
		var s strings.Builder
		for _, node := range nodes {
			if err := node.Render(&s); err != nil {
				return err
			}
		}
		return c.HTML(http.StatusOK, s.String())
	}
}
