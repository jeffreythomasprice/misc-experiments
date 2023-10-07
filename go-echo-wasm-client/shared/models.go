package shared

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type ErrorResponse struct {
	Message string `json:"message"`
}

type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type LoginResponse struct {
	Token string `json:"token"`
}

type errorResponseError struct {
	response     *http.Response
	responseBody ErrorResponse
}

var _ error = &errorResponseError{}

// Error implements error.
func (e *errorResponseError) Error() string {
	return fmt.Sprintf("statusCode=%v, message=%v", e.response.StatusCode, e.responseBody.Message)
}

func MakeJsonRequest[T any](method, url string, requestBody any) (*T, error) {
	var requestBodyReader io.Reader
	if requestBody != nil {
		requestBodyBytes, err := json.Marshal(requestBody)
		if err != nil {
			return nil, err
		}
		requestBodyReader = bytes.NewBuffer(requestBodyBytes)
	}

	request, err := http.NewRequest(method, url, requestBodyReader)
	if err != nil {
		return nil, err
	}
	request.Header.Add("content-type", "application/json")
	request.Header.Add("accept", "application/json")

	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return nil, err
	}

	responseBodyBytes, err := io.ReadAll(response.Body)
	if err != nil {
		return nil, err
	}

	if response.StatusCode >= 200 && response.StatusCode < 300 {
		var result T
		if err := json.Unmarshal(responseBodyBytes, &result); err != nil {
			return nil, err
		}
		return &result, nil
	}

	var result ErrorResponse
	if err := json.Unmarshal(responseBodyBytes, &result); err != nil {
		return nil, err
	}
	return nil, &errorResponseError{
		response:     response,
		responseBody: result,
	}
}
