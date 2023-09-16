package livereload

import (
	"fmt"
	"net/http"
	"time"

	"github.com/go-chi/chi"
	g "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/html"
	"github.com/olahol/melody"

	_ "embed"
)

//go:embed client.js
var script string

func HandleFunc(r chi.Router) {
	m := melody.New()

	r.HandleFunc("/_liveReload", func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	})

	response := []byte(fmt.Sprintf("%d", time.Now().UTC().UnixMilli()))
	m.HandleMessage(func(s *melody.Session, b []byte) {
		s.Write(response)
	})
}

func NewScript() g.Node {
	return Script(g.Raw(script))
}
