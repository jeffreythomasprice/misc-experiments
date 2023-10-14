package main

import (
	"encoding/json"
	"html/template"
	"net/http"
	"strings"
	"time"

	_ "embed"

	"github.com/gin-gonic/gin"
	"github.com/olahol/melody"
	"github.com/rs/zerolog/log"
)

//go:embed assets/embed/liveReload.html
var liveReloadString string

var liveReloadTemplate *template.Template

type templateStringer = func(data any) (string, error)

func init() {
	var err error
	liveReloadTemplate, err = template.New("").Parse(string(liveReloadString))
	if err != nil {
		panic(err)
	}
}

func newTemplateStringer(t *template.Template, name string) templateStringer {
	return func(data any) (string, error) {
		var s strings.Builder
		err := t.ExecuteTemplate(&s, name, nil)
		return s.String(), err
	}
}

type pageRendererOptions struct {
	liveReload bool
}

func pageRenderer(g gin.IRouter, options *pageRendererOptions) func(ctx *gin.Context, f templateStringer, fData any) {
	if options == nil {
		options = &pageRendererOptions{}
	}

	var liveReloadToken *string
	if options.liveReload {
		s := time.Now().Format(time.RFC3339Nano)
		liveReloadToken = &s

		liveReload(g, s)
	}

	return func(ctx *gin.Context, f templateStringer, fData any) {
		fResult, err := f(fData)
		if err != nil {
			ctx.AbortWithError(http.StatusInternalServerError, err)
			return
		}
		content := []template.HTML{
			template.HTML(fResult),
		}

		if liveReloadToken != nil {
			var s strings.Builder
			if err := liveReloadTemplate.Execute(&s, map[string]any{
				"liveReloadToken": *liveReloadToken,
			}); err != nil {
				ctx.AbortWithError(http.StatusInternalServerError, err)
				return
			}

			content = append(content, template.HTML(s.String()))
		}

		ctx.HTML(http.StatusOK, "page.html", map[string]any{
			"content": content,
		})
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
			if err := assetsEmbedTemplates.ExecuteTemplate(&w, "wsReloadWsResponse", nil); err != nil {
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
