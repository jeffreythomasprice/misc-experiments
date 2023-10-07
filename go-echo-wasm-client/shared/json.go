package shared

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
)

// TODO lots of duplicate code

func MakeJsonRequestNoResponse(method, url string, requestBody any) error {
	var requestBodyReader io.Reader
	if requestBody != nil {
		requestBodyBytes, err := json.Marshal(requestBody)
		if err != nil {
			return err
		}
		requestBodyReader = bytes.NewBuffer(requestBodyBytes)
	}

	request, err := http.NewRequest(method, url, requestBodyReader)
	if err != nil {
		return err
	}
	if requestBodyReader != nil {
		request.Header.Add("content-type", "application/json")
	}
	request.Header.Add("accept", "*")

	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return err
	}

	responseBodyBytes, err := io.ReadAll(response.Body)
	if err != nil {
		return err
	}

	if response.StatusCode >= 200 && response.StatusCode < 300 {
		return nil
	}

	var result ErrorResponse
	if err := json.Unmarshal(responseBodyBytes, &result); err != nil {
		return err
	}
	return &HTTPResponseError{
		Response:     response,
		ResponseBody: result,
	}
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
	if requestBodyReader != nil {
		request.Header.Add("content-type", "application/json")
	}
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
	return nil, &HTTPResponseError{
		Response:     response,
		ResponseBody: result,
	}
}
