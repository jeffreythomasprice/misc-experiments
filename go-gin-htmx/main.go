package main

import (
	"embed"
	"html/template"
	"io/fs"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/olahol/melody"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

//go:embed assets/embed/*
var embeddedTemplateAssets embed.FS

//go:embed assets/*.js
var embeddedStaticAssets embed.FS

func main() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	zerolog.SetGlobalLevel(zerolog.TraceLevel)
	log.Logger = log.Output(zerolog.ConsoleWriter{
		Out:        os.Stdout,
		TimeFormat: time.RFC3339Nano,
	})

	timestamp := time.Now().Format(time.RFC3339Nano)
	page := func(ctx *gin.Context, f TemplateStringer, fData any) {
		content, err := f(fData)
		if err != nil {
			ctx.AbortWithError(http.StatusInternalServerError, err)
			return
		}
		ctx.HTML(http.StatusOK, "page.html", map[string]any{
			"content":   template.HTML(content),
			"timestamp": timestamp,
		})
	}

	// TODO customize gin logger
	g := gin.Default()

	{
		templs, err := template.ParseFS(embeddedTemplateAssets, "assets/embed/*")
		if err != nil {
			panic(err)
		}
		g.SetHTMLTemplate(templs)
		g.GET("/", func(ctx *gin.Context) {
			page(ctx, templateStringer(templs, "clicks.html"), nil)
		})
	}

	{
		x, err := fs.Sub(embeddedStaticAssets, "assets")
		if err != nil {
			panic(err)
		}
		g.StaticFS("/assets", http.FS(x))
	}

	clicks := 0
	g.GET("/click", func(ctx *gin.Context) {
		ctx.HTML(http.StatusOK, "clickResults.html", clicks)
	})
	g.POST("/click", func(ctx *gin.Context) {
		clicks++
		ctx.HTML(http.StatusOK, "clickResults.html", clicks)
	})

	m := melody.New()
	g.GET("/ws", gin.WrapF(func(w http.ResponseWriter, r *http.Request) {
		log := log.With().Str("remote addr", r.RemoteAddr).Logger()
		log.Debug().Msg("websocket connected")
		if err := m.HandleRequest(w, r); err != nil {
			// TODO context about remote addr
			log.Error().Err(err).Msg("error handling websocket")
		}
		log.Debug().Msg("websocket disconnected")
	}))
	m.HandleMessage(func(s *melody.Session, b []byte) {
		log.Debug().Str("b", string(b)).Msg("text message")
	})

	g.Run("127.0.0.1:8000")
}

type TemplateStringer = func(data any) (string, error)

func templateStringer(t *template.Template, name string) TemplateStringer {
	return func(data any) (string, error) {
		var s strings.Builder
		err := t.ExecuteTemplate(&s, name, nil)
		return s.String(), err
	}
}
