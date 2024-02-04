package views

import (
	"context"
	_ "embed"
	"html/template"
	"io"
	"strings"
	"sync"

	"github.com/rs/zerolog"
)

type templateFunc func(ctx context.Context, w io.Writer, data any) error

//go:embed page.html
var pageSource string
var pageFunc = snippet(pageSource, "")

var stringBuilders sync.Pool = sync.Pool{
	New: func() any {
		return new(strings.Builder)
	},
}

func page(source, name string) templateFunc {
	var snippetFunc templateFunc
	return func(ctx context.Context, w io.Writer, data any) error {
		if snippetFunc == nil {
			snippetFunc = snippet(source, name)
		}
		var buf *strings.Builder = stringBuilders.Get().(*strings.Builder)
		defer stringBuilders.Put(buf)
		buf.Reset()
		if err := snippetFunc(ctx, buf, data); err != nil {
			return err
		}
		return pageFunc(ctx, w, template.HTML(buf.String()))
	}
}

func snippet(source, name string) templateFunc {
	var t *template.Template
	return func(ctx context.Context, w io.Writer, data any) error {
		var err error
		if t == nil {
			t, err = template.New("").Parse(source)
			if err != nil {
				log := zerolog.Ctx(ctx)
				log.Fatal().
					Err(err).
					Msg("failed to parse template")
			}
		}

		return t.ExecuteTemplate(w, name, data)
	}
}
