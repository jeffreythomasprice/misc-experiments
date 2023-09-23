package shared

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net/http"
)

func MakeJSONRequest[T any](method string, url string, requestBody interface{}) (*T, error) {
	slog.Debug(
		"request",
		"method", method,
		"url", url,
		"body", requestBody,
	)
	var c http.Client
	var buf bytes.Buffer
	if err := MarshalJson(&buf, requestBody); err != nil {
		return nil, fmt.Errorf("error serializing request %w", err)
	}
	request, err := http.NewRequest(method, url, &buf)
	if err != nil {
		return nil, fmt.Errorf("error making request %w", err)
	}
	request.Header.Set("content-type", "application/json")
	request.Header.Set("accept", "application/json")
	response, err := c.Do(request)
	if err != nil {
		return nil, fmt.Errorf("error sending request: %w", err)
	}
	responseBody, err := io.ReadAll(response.Body)
	if err != nil {
		return nil, fmt.Errorf("error reading response body: %w", err)
	}
	slog.Debug(
		"response",
		"status", response.Status,
		"header", response.Header,
		"body", string(responseBody),
	)
	var result T
	err = json.Unmarshal(responseBody, &result)
	return &result, err
}

func MarshalJson(w io.Writer, value interface{}) error {
	return json.NewEncoder(w).Encode(value)
}

func UnmarshalJson[T any](r io.Reader) (*T, error) {
	var result T
	err := json.NewDecoder(r).Decode(&result)
	if errors.Is(err, io.EOF) {
		err = nil
	}
	return &result, err
}
