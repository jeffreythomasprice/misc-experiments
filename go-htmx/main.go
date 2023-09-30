package main

import (
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"sync"

	"github.com/go-chi/chi/v5"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/components"
	. "github.com/maragudk/gomponents/html"
)

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level:     slog.LevelDebug,
			AddSource: true,
		},
	)))

	mux := chi.NewRouter()

	mux.Get("/", newHtmlHandlerFunc(func(w http.ResponseWriter, r *http.Request) ([]Node, error) {
		// TODO if request had valid auth cookie render the logged in page
		return []Node{HTML5(HTML5Props{
			Title: "htmx experiment",
			Head: []Node{
				Script(Src("https://unpkg.com/htmx.org@1.9.6")),
				Script(Text("htmx.logAll()")),
			},
			Body: notLoggedIn(),
		})}, nil
	}))

	mux.Post("/login", newHtmlHandlerFunc(func(w http.ResponseWriter, r *http.Request) ([]Node, error) {
		if err := r.ParseForm(); err != nil {
			return loginFormError("form parse error"), nil
		}

		username, ok := r.Form["username"]
		if !ok || len(username) != 1 {
			return loginFormError("malformed username"), nil
		}

		password, ok := r.Form["password"]
		if !ok || len(password) != 1 {
			return loginFormError("malformed password"), nil
		}

		slog.Debug("TODO JEFF login form", "username", username[0], "password", password[0])

		return loginFormError("TODO err msg"), nil
	}))

	bindAddr := "127.0.0.1:8000"
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		if err := http.ListenAndServe(bindAddr, mux); err != nil {
			slog.Error("server failed with error", "err", err)
		}
	}()
	slog.Info("server started", "bindAddr", bindAddr)
	wg.Wait()
	slog.Debug("exiting")
}

func notLoggedIn() []Node {
	return []Node{
		Div(
			Label(
				Text("Username:"),
				For("username"),
			),
			Input(
				Name("username"),
				FormAttr("loginForm"),
				Type("text"),
				Placeholder("Username"),
			),
		),
		Div(
			Label(
				Text("Password:"),
				For("password"),
			),
			Input(
				Name("password"),
				FormAttr("loginForm"),
				Type("password"),
				Placeholder("Password"),
			),
		),
		FormEl(
			ID("loginForm"),
			Attr("hx-post", "/login"),
			loginButton(),
		),
		Div(ID("loginErrors")),
	}
}

func loginFormError(s string) []Node {
	slog.Info("login form error", "msg", s)
	return []Node{
		loginButton(),
		Div(
			ID("loginErrors"),
			Attr("hx-swap-oob", "outerHTML"),
			StyleAttr("color: red; font-weight: bold"),
			Text(s),
		),
	}
}

func loginButton() Node {
	return Button(
		Text("Click Me"),
		Type("submit"),
	)
}

func newHtmlHandlerFunc(f func(w http.ResponseWriter, r *http.Request) ([]Node, error)) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		nodes, err := f(w, r)
		if err != nil {
			writeErrorResponse(w, err, 500, "internal server error")
			return
		}

		var s strings.Builder
		for _, child := range nodes {
			if err := child.Render(&s); err != nil {
				writeErrorResponse(w, err, 500, "internal server error")
				return
			}
		}

		w.Header().Add("content-type", "text/html")
		_, err = fmt.Fprint(w, s.String())
		if err != nil {
			slog.Error("error writing content to http writer", "err", err)
		}
	}
}

func writeErrorResponse(w http.ResponseWriter, err error, statusCode int, message string) {
	slog.Error(
		"error response",
		"err", err,
		"statusCode", statusCode,
	)
	w.WriteHeader(statusCode)
	_, err = fmt.Fprint(w, message)
	if err != nil {
		slog.Error(
			"an error occurred writing an error message in response to a previous error",
			"err", err,
			"statusCode", statusCode,
		)
	}
}
