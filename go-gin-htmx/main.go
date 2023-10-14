package main

import (
	"embed"
	"encoding/json"
	"html/template"
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

	gin.SetMode(gin.ReleaseMode)
	g := gin.New()
	g.Use(gin.LoggerWithFormatter(func(params gin.LogFormatterParams) string {
		log.Debug().
			Int("status code", params.StatusCode).
			Str("latency", params.Latency.String()).
			Str("client IP", params.ClientIP).
			Str("method", params.Method).
			Str("path", params.Path).
			Msg("gin")
		return ""
	}))
	g.Use(gin.Recovery())

	templs, err := template.ParseFS(embeddedTemplateAssets, "assets/embed/*")
	if err != nil {
		panic(err)
	}
	g.SetHTMLTemplate(templs)
	g.GET("/", func(ctx *gin.Context) {
		page(ctx, templateStringer(templs, "clicks.html"), nil)
	})

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
			log.Error().Err(err).Msg("error handling websocket")
		}
		log.Debug().Msg("websocket disconnected")
	}))
	m.HandleMessage(func(s *melody.Session, b []byte) {
		log := log.With().Str("remote addr", s.Request.RemoteAddr).Logger()

		var message struct {
			Timestamp string `json:"timestamp"`
		}
		if err := json.Unmarshal(b, &message); err != nil {
			log.Error().
				Err(err).
				Msg("error parsing websocket message")
			return
		}

		if timestamp != message.Timestamp {
			log.Info().Msg("client is out of date, sending refresh")
			// TODO send refresh
		}
	})

	addr := "127.0.0.1:8000"
	log.Info().Str("addr", addr).Msg("starting server")
	if err := g.Run(addr); err != nil {
		log.Error().Err(err).Msg("server error")
	}
}

type TemplateStringer = func(data any) (string, error)

func templateStringer(t *template.Template, name string) TemplateStringer {
	return func(data any) (string, error) {
		var s strings.Builder
		err := t.ExecuteTemplate(&s, name, nil)
		return s.String(), err
	}
}
