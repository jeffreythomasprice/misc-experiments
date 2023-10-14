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

var embeddedTemplateAssetsTemplates *template.Template

func init() {
	var err error
	embeddedTemplateAssetsTemplates, err = template.ParseFS(embeddedTemplateAssets, "assets/embed/*")
	if err != nil {
		panic(err)
	}
}

func main() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	zerolog.SetGlobalLevel(zerolog.TraceLevel)
	log.Logger = log.Output(zerolog.ConsoleWriter{
		Out:        os.Stdout,
		TimeFormat: time.RFC3339Nano,
	})

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

	g.SetHTMLTemplate(embeddedTemplateAssetsTemplates)

	liveReloadToken := time.Now().Format(time.RFC3339Nano)
	page := func(ctx *gin.Context, f TemplateStringer, fData any) {
		content, err := f(fData)
		if err != nil {
			ctx.AbortWithError(http.StatusInternalServerError, err)
			return
		}
		ctx.HTML(http.StatusOK, "page.html", map[string]any{
			"content":         template.HTML(content),
			"liveReloadToken": liveReloadToken,
		})
	}

	liveReload(g, liveReloadToken)

	g.GET("/", func(ctx *gin.Context) {
		page(ctx, templateStringer(embeddedTemplateAssetsTemplates, "clicks.html"), nil)
	})

	clicks(g)

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

func liveReload(r gin.IRouter, token string) {
	m := melody.New()

	m.HandleMessage(func(s *melody.Session, b []byte) {
		log := log.With().Str("remote addr", s.Request.RemoteAddr).Logger()

		var message struct {
			Token string `json:"liveReloadToken"`
		}
		if err := json.Unmarshal(b, &message); err != nil {
			log.Error().
				Err(err).
				Msg("error parsing websocket message")
			return
		}

		if token != message.Token {
			log.Info().Msg("client is out of date, sending refresh")
			var w strings.Builder
			if err := embeddedTemplateAssetsTemplates.ExecuteTemplate(&w, "wsReloadWsResponse", nil); err != nil {
				log.Error().Err(err).Msg("error rendering websocket reload")
			} else {
				err := s.Write([]byte(w.String()))
				if err != nil {
					log.Error().Err(err).Msg("error writing websocket reload message")
				}
			}
		}
	})

	r.GET("/liveReload/ws", gin.WrapF(func(w http.ResponseWriter, r *http.Request) {
		log := log.With().Str("remote addr", r.RemoteAddr).Logger()
		log.Debug().Msg("websocket connected")
		if err := m.HandleRequest(w, r); err != nil {
			log.Error().Err(err).Msg("error handling websocket")
		}
		log.Debug().Msg("websocket disconnected")
	}))

	r.GET("/liveReload/trigger", func(ctx *gin.Context) {
		ctx.Header("hx-refresh", "true")
		ctx.String(http.StatusOK, "")
	})
}

func clicks(r gin.IRouter) {
	clicks := 0

	r.GET("/click", func(ctx *gin.Context) {
		ctx.HTML(http.StatusOK, "clickResults.html", clicks)
	})

	r.POST("/click", func(ctx *gin.Context) {
		clicks++
		ctx.HTML(http.StatusOK, "clickResults.html", clicks)
	})
}
