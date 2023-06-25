package main

import (
	"context"
	"embed"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"regexp"
	"strings"
	"syscall"

	"go.uber.org/zap"
)

//go:embed schemas
var schemas embed.FS

// TODO JEFF testing
type ExampleRequest struct {
	Foo string `json:"foo"`
	Bar int    `json:"bar"`
}

type ExampleResponse struct {
	Foo       int
	Bar       string
	PathParam string `json:"pathParam"`
}

func main() {
	log, closeLogger, err := createLogger()
	if err != nil {
		panic(err)
	}
	defer closeLogger()

	jsonSchemaValidator := NewJsonSchemaValidator()
	if err := jsonSchemaValidator.AddSchemaDirectory(schemas, "."); err != nil {
		log.Panic("failed to add dir", zap.Error(err))
	}

	router := NewRouterBuilder(log)

	// TODO JEFF testing
	router.Get(
		regexp.MustCompile("^/$"),
		func(log *zap.Logger, response http.ResponseWriter, request *RouteMatchedRequest) error {
			response.WriteHeader(200)
			fmt.Fprintf(response, "Hello, World!\n")
			return nil
		},
	)

	// TODO JEFF testing
	router.Get(
		regexp.MustCompile("^/teapot$"),
		func(log *zap.Logger, response http.ResponseWriter, request *RouteMatchedRequest) error {
			response.WriteHeader(418)
			fmt.Fprintf(response, "I'm a teapot!\n")
			return nil
		},
	)

	// TODO JEFF testing
	{
		f, err := NewJsonSchemaParserFunc[ExampleRequest](jsonSchemaValidator, "schemas/example-request.json")
		if err != nil {
			log.Panic("failed to make validator", zap.Error(err))
		}
		router.Post(
			regexp.MustCompile(`^/([a-zA-Z0-9_]+)$`),
			f.RouteFunc(func(log *zap.Logger, response http.ResponseWriter, request *RouteMatchedRequest, requestBody ExampleRequest) error {
				pathParam := request.pathSubmatch[1]
				log.Debug("post request with path param", zap.String("path param", pathParam))
				return json.NewEncoder(response).Encode(ExampleResponse{
					PathParam: pathParam,
					Foo:       requestBody.Bar,
					Bar:       strings.ToUpper(requestBody.Foo),
				})
			}),
		)
	}

	const addr = "127.0.0.1:8000"
	server := &http.Server{
		Addr:    addr,
		Handler: router.Build(),
	}
	go func() {
		log.Info("starting server", zap.String("addr", addr))
		if err := server.ListenAndServe(); err != nil {
			if !errors.Is(err, http.ErrServerClosed) {
				log.Error("server error", zap.String("addr", addr), zap.Error(err))
			}
		}
	}()

	ctx, cancel := signal.NotifyContext(
		context.Background(),
		// ctrl-c
		os.Interrupt,
		// nodemon, used by watch script?
		syscall.SIGUSR2,
	)
	defer cancel()
	<-ctx.Done()
	log.Info("shutting down")
	if err := server.Shutdown(context.Background()); err != nil {
		log.Error("server shutdown error", zap.Error(err))
	}
}
