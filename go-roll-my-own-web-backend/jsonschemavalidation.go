package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"net/http"

	"github.com/santhosh-tekuri/jsonschema"
	"go.uber.org/zap"
)

type JsonSchemaParserFunc[T any] func(r io.Reader, result *T) error

type JsonSchemaValidator struct {
	contents map[string]*jsonschema.Schema
	compiler *jsonschema.Compiler
}

type JsonRequestRouteFunc[T any] func(log *zap.Logger, response http.ResponseWriter, request *RouteMatchedRequest, requestBody T) error

func NewJsonSchemaValidator() *JsonSchemaValidator {
	return &JsonSchemaValidator{
		contents: make(map[string]*jsonschema.Schema),
		compiler: jsonschema.NewCompiler(),
	}
}

func (validator *JsonSchemaValidator) Get(url string) (*jsonschema.Schema, bool) {
	result, exists := validator.contents[url]
	return result, exists
}

func (validator *JsonSchemaValidator) AddSchema(url string, r io.Reader) (*jsonschema.Schema, error) {
	if _, exists := validator.contents[url]; exists {
		return nil, fmt.Errorf("duplicate key: %v", url)
	}
	if err := validator.compiler.AddResource(url, r); err != nil {
		return nil, err
	}
	result, err := validator.compiler.Compile(url)
	if err != nil {
		return nil, err
	}
	validator.contents[url] = result
	return result, nil
}

func (validator *JsonSchemaValidator) AddSchemaDirectory(files fs.ReadDirFS, dir string) error {
	dirEntry, err := getAllFilesRecursively(files, ".")
	if err != nil {
		return fmt.Errorf("failed to read dir: %v\n%w", dir, err)
	}
	for _, e := range dirEntry {
		r, err := e.Open()
		if err != nil {
			return fmt.Errorf("failed to open file: %v\n%w", e.Path(), err)
		}
		defer r.Close()
		if _, err := validator.AddSchema(e.Path(), r); err != nil {
			return fmt.Errorf("failed to add schema resource: %v\n%w", e.Path(), err)
		}
	}
	return nil
}

func NewJsonSchemaParserFunc[T any](validator *JsonSchemaValidator, url string) (JsonSchemaParserFunc[T], error) {
	schema, exists := validator.Get(url)
	if !exists {
		return nil, fmt.Errorf("no such schema loaded: %v", url)
	}

	// TODO make buffer a shared thing so these become thread safe?
	buffer := make([]byte, 1024)
	return func(r io.Reader, result *T) error {
		totalLen := 0
		for {
			n, err := r.Read(buffer)
			if err != nil && !errors.Is(err, io.EOF) {
				return err
			}
			if n == 0 {
				break
			}
			totalLen += n
			if n == len(buffer) {
				newBuffer := make([]byte, len(buffer)+1024)
				copy(newBuffer, buffer)
				buffer = newBuffer
			}
		}
		bufferSlice := buffer[:totalLen]

		if err := schema.Validate(bytes.NewBuffer(bufferSlice)); err != nil {
			return err
		}

		decoder := json.NewDecoder(bytes.NewBuffer(bufferSlice))
		if err := decoder.Decode(result); err != nil {
			return err
		}
		return nil
	}, nil
}

func (parser JsonSchemaParserFunc[T]) RouteFunc(f JsonRequestRouteFunc[T]) RouteFunc {
	return func(log *zap.Logger, response http.ResponseWriter, request *RouteMatchedRequest) error {
		var requestBody T
		if err := parser(request.Body, &requestBody); err != nil {
			log.Debug("request failed validation", zap.Error(err))

			// validation failed because there was no payload
			if errors.Is(err, io.EOF) {
				if request.ContentLength == 0 {
					RespondWithErrorObject(response, request.Request, http.StatusBadRequest, "expected request body")
				} else {
					// or maybe an unexpected error occurred reading, but this ought to be impossible
					RespondWithErrorObject(response, request.Request, http.StatusBadRequest, "unexpected EOF")
				}
				return nil
			}

			// thye just gave us bad data
			var validationError *jsonschema.ValidationError
			if errors.As(err, &validationError) {
				RespondWithErrorObject(response, request.Request, http.StatusBadRequest, validationError)
				return nil
			}

			// any other kind of error can be the generic error response
			return err
		}
		return f(log, response, request, requestBody)
	}
}
