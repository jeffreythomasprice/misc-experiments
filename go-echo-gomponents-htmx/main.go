package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	glog "github.com/labstack/gommon/log"
	"github.com/rs/zerolog/log"
	"github.com/ziflex/lecho/v3"
	"golang.org/x/net/websocket"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"

	"experiment/utils"

	g "github.com/maragudk/gomponents"
	c "github.com/maragudk/gomponents/components"
	h "github.com/maragudk/gomponents/html"
)

func main() {
	zlog := utils.ZerologInitPretty()
	elog := lecho.From(zlog, lecho.WithLevel(glog.INFO))

	_, err := gorm.Open(sqlite.Open("local.db"), &gorm.Config{})
	if err != nil {
		log.Panic().Err(err).Msg("failed to open db")
	}

	e := echo.New()
	e.Logger = elog
	e.Use(middleware.RequestID())
	e.Use(lecho.Middleware(lecho.Config{Logger: elog}))

	e.HideBanner = true

	clicks := 0

	e.GET("/", func(c echo.Context) error {
		return renderComponentToResponse(c, index(
			clickResults(clicks),
			h.Button(
				g.Attr("hx-post", "/click"),
				g.Attr("hx-target", "#clicks"),
				g.Text("Click Me"),
			),

			h.Div(
				g.Attr("hx-ws", "connect:/ws"),
				h.Div(h.ID("wsOutput")),
				h.FormEl(
					g.Attr("hx-ws", "send"),
					h.Input(
						h.Type("type"),
						h.Name("text"),
						h.Placeholder("Type a message here"),
						h.AutoFocus(),
						h.AutoComplete("off"),
					),
				),
			),
		))
	})

	e.POST("/click", func(c echo.Context) error {
		clicks++
		return renderComponentToResponse(c, clickResults(clicks))
	})

	activeWebsockets := make(map[string]*websocket.Conn)
	var activeWebsocketsMutex sync.Mutex
	var nextId atomic.Int32
	nextId.Store(0)
	e.GET("/ws", func(c echo.Context) error {
		websocket.Handler(func(c *websocket.Conn) {
			defer c.Close()

			id := fmt.Sprintf("ws-%v", nextId.Add(1))
			activeWebsocketsMutex.Lock()
			activeWebsockets[id] = c
			activeWebsocketsMutex.Unlock()

			log := log.With().
				Str("id", id).
				Str("remote addr", c.Request().RemoteAddr).
				Logger()
			log.Info().Msg("new websocket connection")

			var wg sync.WaitGroup

			wg.Add(1)
			go func() {
				defer wg.Done()
				for {
					msg := ""
					if err := websocket.Message.Receive(c, &msg); err != nil {
						if errors.Is(err, net.ErrClosed) || errors.Is(err, io.EOF) {
							log.Trace().Msg("existing receive loop because websocket is closed")
						} else {
							log.Err(err).Msg("websocket receive error")
						}
						return
					}
					log.Trace().Str("received", msg).Msg("received from websocket")

					type WSMessage struct {
						Text string `json:"text"`
					}
					var wsMessage WSMessage
					if err := json.Unmarshal([]byte(msg), &wsMessage); err != nil {
						log.Err(err).Msg("failed to unmarshal websocket message")
					}

					activeWebsocketsMutex.Lock()
					for _, c := range activeWebsockets {
						if err := renderComponentToWebsocket(
							c,
							h.Div(
								h.ID("wsOutput"),
								g.Attr("hx-swap-oob", "beforeend"),
								h.Div(
									g.Textf("%v %v %v", time.Now().Format(time.RFC3339), id, wsMessage.Text),
								),
							),
						); err != nil {
							log.Err(err).Msg("websocket send error")
						}
					}
					activeWebsocketsMutex.Unlock()
				}
			}()

			wg.Wait()

			delete(activeWebsockets, id)
			log.Info().Msg("websocket closed")
		}).
			ServeHTTP(c.Response(), c.Request())
		return nil
	})

	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

func clickResults(clicks int) g.Node {
	return h.Div(
		h.ID("clicks"),
		g.Textf("Clicks: %v", clicks),
	)
}

func index(body ...g.Node) g.Node {
	return c.HTML5(c.HTML5Props{
		Title:    "",
		Language: "en",
		Head: []g.Node{
			h.Script(h.Src("https://unpkg.com/htmx.org@1.9.10")),
			h.Script(g.Text("htmx.logAll()")),
		},
		Body: body,
	})
}

func renderComponentToResponse(c echo.Context, components ...g.Node) error {
	for _, comp := range components {
		if err := comp.Render(c.Response()); err != nil {
			return err
		}
	}
	return nil
}

func renderComponentToWebsocket(c *websocket.Conn, components ...g.Node) error {
	var s strings.Builder
	for _, comp := range components {
		if err := comp.Render(&s); err != nil {
			return err
		}
	}
	if err := websocket.Message.Send(c, s.String()); err != nil {
		return err
	}
	return nil
}
