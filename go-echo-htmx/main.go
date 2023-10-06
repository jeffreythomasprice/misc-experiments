package main

import (
	"errors"
	"log/slog"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	slogecho "github.com/samber/slog-echo"

	. "github.com/maragudk/gomponents"
	. "github.com/maragudk/gomponents/components"
	. "github.com/maragudk/gomponents/html"
)

type usersService struct{}

func (service *usersService) ValidateCredentials(username, password string) (bool, error) {
	slog.Debug("checking credentials", "username", username)
	result := password == "password"
	slog.Debug("result", "username", username, "result", result)
	return result, nil
}

func (service *usersService) IsLoggedInCookie(cookie *http.Cookie, err error) (bool, error) {
	if err != nil {
		if errors.Is(err, http.ErrNoCookie) {
			slog.Debug("missing auth cookie")
			return false, nil
		}
		return false, err
	}
	slog.Debug("TODO should check auth cookie", "cookie", cookie)
	return true, nil
}

type serviceContext struct {
	echo.Context
	users *usersService
}

func main() {
	slog.SetDefault(slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level:     slog.LevelDebug,
		AddSource: true,
	})))

	e := echo.New()
	e.HideBanner = true
	e.HidePort = true
	e.Debug = true

	e.Use(slogecho.New(slog.Default()))
	e.Use(middleware.Recover())

	users := &usersService{}
	e.Use(func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			cc := &serviceContext{
				Context: c,
				users:   users,
			}
			return next(cc)
		}
	})

	e.GET("/", htmlNodeHandler(func(c *serviceContext) (Node, error) {
		isLoggedIn, err := c.users.IsLoggedInCookie(c.Cookie("auth"))
		if err != nil {
			return nil, err
		}
		if isLoggedIn {
			return page(c, loggedIn)
		} else {
			return page(c, notLoggedIn)
		}
	}))

	e.POST("/api/login", htmlNodeHandler(login))

	addr := "127.0.0.1:8000"
	slog.Info("listening", "addr", addr)
	e.Logger.Fatal(e.Start(addr))
}

func notLoggedIn(c *serviceContext) (Node, error) {
	return Group([]Node{
		FormEl(
			Attr("hx-post", "/api/login"),
			Attr("hx-swap", "none"),
			Div(
				Label(
					For("username"),
					Text("Username:"),
				),
				Input(
					Name("username"),
					Placeholder("Username"),
					Type("text"),
				),
			),
			Div(
				Label(
					For("password"),
					Text("Password:"),
				),
				Input(
					Name("password"),
					Placeholder("Password"),
					Type("password"),
				),
			),
			Button(
				Type("submit"),
				Text("Log In"),
			),
		),
		Div(ID("errorMessages")),
	}), nil
}

func loggedIn(c *serviceContext) (Node, error) {
	return Div(Text("TODO logged in page")), nil
}

func login(c *serviceContext) (Node, error) {
	var request struct {
		Username string `form:"username"`
		Password string `form:"password"`
	}
	if err := c.Bind(&request); err != nil {
		return nil, err
	}

	slog.Debug("login", "request", request)

	if len(request.Username) == 0 {
		return errorMessage(c, "Username is required")
	}

	if len(request.Password) == 0 {
		return errorMessage(c, "Password is required")
	}

	isValid, err := c.users.ValidateCredentials(request.Username, request.Password)
	if err != nil {
		return nil, err
	}
	if !isValid {
		return errorMessage(c, "Invalid credentials")
	}

	authCookie := new(http.Cookie)
	authCookie.Name = "auth"
	authCookie.Value = "TODO jwt here"
	// TODO expire should match jwt
	authCookie.Expires = time.Now().Add(60 * time.Second)
	authCookie.Path = "/"
	c.SetCookie(authCookie)

	c.Response().Header().Add("hx-refresh", "true")
	return loggedIn(c)
}

func errorMessage(c *serviceContext, msg string) (Node, error) {
	return Div(
		Attr("hx-swap-oob", "innerHTML:#errorMessages"),
		Div(
			Class("error"),
			Text(msg),
		),
	), nil
}

func page(c *serviceContext, content func(c *serviceContext) (Node, error)) (Node, error) {
	contentNode, err := content(c)
	if err != nil {
		return nil, err
	}
	return HTML5(HTML5Props{
		Head: []Node{
			StyleEl(Text(`
				.error {
					font-weight: bold;
					color: red;
				}
			`)),
			Script(
				Src("https://unpkg.com/htmx.org@1.9.6"),
				Text("htmx.logAll();"),
			),
		},
		Body: []Node{contentNode},
	}), nil
}

func htmlNodeHandler(f func(c *serviceContext) (Node, error)) echo.HandlerFunc {
	return func(c echo.Context) error {
		node, err := f(c.(*serviceContext))
		if err != nil {
			return err
		}
		var s strings.Builder
		if err := node.Render(&s); err != nil {
			return err
		}
		return c.HTML(http.StatusOK, s.String())
	}
}
