package main

import (
	"github.com/a-h/templ"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	glog "github.com/labstack/gommon/log"
	"github.com/rs/zerolog/log"
	"github.com/ziflex/lecho/v3"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"

	"experiment/database"
	"experiment/utils"
	"experiment/views/messages"
)

func main() {
	zlog := utils.ZerologInitPretty()
	elog := lecho.From(zlog, lecho.WithLevel(glog.INFO))

	db, err := gorm.Open(sqlite.Open("local.db"), &gorm.Config{})
	if err != nil {
		log.Panic().Err(err).Msg("failed to open db")
	}
	if err := database.Init(db); err != nil {
		log.Panic().Err(err).Msg("failed to init db")
	}

	e := echo.New()
	e.Logger = elog
	e.Use(middleware.RequestID())
	e.Use(lecho.Middleware(lecho.Config{Logger: elog}))

	e.HideBanner = true

	convertDBMessageToViewMessage := func(input *database.Message) *messages.Message {
		return &messages.Message{
			ID:      input.ID,
			Message: input.Message,
		}
	}

	getMessages := func() ([]*messages.Message, error) {
		dbMessages, err := database.ListMessages(db)
		if err != nil {
			return nil, err
		}
		results := make([]*messages.Message, len(dbMessages))
		for i, msg := range dbMessages {
			results[i] = convertDBMessageToViewMessage(msg)
		}
		return results, nil
	}

	e.GET("/", func(c echo.Context) error {
		results, err := getMessages()
		if err != nil {
			// TODO return error document
			return err
		}

		return templRenderToEcho(index(results), c)
	})

	e.POST("/messages", func(c echo.Context) error {
		type Request struct {
			Message string `form:"message"`
		}
		request := new(Request)
		if err := c.Bind(request); err != nil {
			// TODO return error document
			return err
		}
		log.Debug().
			Str("payload", request.Message).
			Msg("creating new message")

		// TODO add new message to db
		if _, err := database.CreateMessage(db, &database.Message{Message: request.Message}); err != nil {
			// TODO return error document
			return err
		}

		results, err := getMessages()
		if err != nil {
			// TODO return error document
			return err
		}

		return templRenderToEcho(messages.MessagesListAndForm(results), c)
	})

	e.GET("/messages/:id", func(c echo.Context) error {
		type Request struct {
			ID uint `param:"id"`
		}
		request := new(Request)
		if err := c.Bind(request); err != nil {
			// TODO return error document
			return err
		}
		log.Debug().
			Uint("id", request.ID).
			Msg("getting message update form")

		msg, err := database.GetMessage(db, request.ID)
		if err != nil {
			// TODO return error document
			return err
		}

		return templRenderToEcho(messages.MessageEditForm(convertDBMessageToViewMessage(msg)), c)
	})

	e.PUT("/messages/:id", func(c echo.Context) error {
		type Request struct {
			ID      uint   `param:"id"`
			Message string `form:"message"`
		}
		request := new(Request)
		if err := c.Bind(request); err != nil {
			// TODO return error document
			return err
		}
		log.Debug().
			Uint("id", request.ID).
			Str("payload", request.Message).
			Msg("updating message")

		if _, err := database.UpdateMessage(db, &database.Message{ID: request.ID, Message: request.Message}); err != nil {
			// TODO return error document
			return err
		}

		results, err := getMessages()
		if err != nil {
			// TODO return error document
			return err
		}

		return templRenderToEcho(messages.MessagesListAndForm(results), c)
	})

	e.DELETE("/messages/:id", func(c echo.Context) error {
		type Request struct {
			ID uint `param:"id"`
		}
		request := new(Request)
		if err := c.Bind(request); err != nil {
			// TODO return error document
			return err
		}
		log.Debug().
			Uint("id", request.ID).
			Msg("deleting message")

		if err := database.DeleteMessage(db, request.ID); err != nil {
			// TODO return error document
			return err
		}

		results, err := getMessages()
		if err != nil {
			// TODO return error document
			return err
		}

		return templRenderToEcho(messages.MessagesListAndForm(results), c)
	})

	e.Logger.Fatal(e.Start("127.0.0.1:8000"))
}

func templRenderToEcho(comp templ.Component, ctx echo.Context) error {
	return comp.Render(ctx.Request().Context(), ctx.Response())
}
