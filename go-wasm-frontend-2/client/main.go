package main

import (
	"client/dom"
	"fmt"
	"log/slog"
	"shared"
)

func main() {
	shared.InitSlog()

	go liveReload("ws://localhost:8000/_liveReload")

	must(dom.Append(dom.QuerySelector("body"), func() (dom.Rendered, error) {
		return dom.DomString(`
			<div id="message">Hello, World! 2</div>
			<button>Click Me</button>
		`)
	}))

	count := 0
	must(dom.OnClick(dom.QuerySelector("button"), func(e dom.Event) {
		slog.Debug("clicked!")

		must(dom.Replace(dom.QuerySelector("#message"), func() (dom.Rendered, error) {
			count++
			return dom.DomString(fmt.Sprintf(`
				<div id="message">%v</div>
			`, count))
		}))
	}))

	select {}
}

func must(err error) {
	if err != nil {
		panic(err)
	}
}
