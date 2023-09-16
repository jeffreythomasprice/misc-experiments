package livereload

import (
	"fmt"
	"net/http"
	"strings"
	"text/template"
	"time"

	"github.com/go-chi/chi"
	g "github.com/maragudk/gomponents"
	h "github.com/maragudk/gomponents/html"
	"github.com/olahol/melody"

	_ "embed"
)

type scriptData struct {
	Path string
}

//go:embed client.js
var script string
var scriptTemplate *template.Template

func init() {
	var err error
	scriptTemplate, err = template.New("").Parse(script)
	if err != nil {
		panic(err)
	}
}

func HandlerFunc(r chi.Router) http.HandlerFunc {
	m := melody.New()

	response := []byte(fmt.Sprintf("%d", time.Now().UTC().UnixMilli()))
	m.HandleConnect(func(s *melody.Session) {
		s.Write(response)
	})

	return func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	}
}

func Script(path string) (g.Node, error) {
	var w strings.Builder
	if err := scriptTemplate.Execute(&w, &scriptData{Path: path}); err != nil {
		return nil, err
	}
	return h.Script(g.Raw(w.String())), nil
}
