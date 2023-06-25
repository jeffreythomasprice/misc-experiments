package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"golang.org/x/exp/slices"
)

type genericErrorResponse struct {
	Message string `json:"message"`
}

func Accepts(request *http.Request, requiredMimeType string) bool {
	allAcceptsHeaders := request.Header.Values("accepts")
	if len(allAcceptsHeaders) == 0 {
		return true
	}
	requiredMimeType = strings.ToLower(requiredMimeType)
	return slices.ContainsFunc[string](allAcceptsHeaders, func(s string) bool {
		s = strings.ToLower(s)
		if s == "*/*" {
			return true
		}
		if s == requiredMimeType {
			return true
		}
		return false
	})
}

func RespondWithErrorObject(response http.ResponseWriter, request *http.Request, statusCode int, errResponse interface{}) error {
	const applicationJson = "application/json"
	const textPlain = "text/plain"

	if Accepts(request, textPlain) {
		response.Header().Add("Content-Type", textPlain)
		response.WriteHeader(statusCode)
		_, err := fmt.Fprintf(response, "%v", errResponse)
		return err
	}

	if Accepts(request, applicationJson) {
		response.Header().Add("Content-Type", applicationJson)
		response.WriteHeader(statusCode)
		return json.NewEncoder(response).Encode(errResponse)
	}

	response.Header().Add("Content-Type", "text/plain")
	statusCode = http.StatusUnsupportedMediaType
	response.WriteHeader(statusCode)
	_, err := fmt.Fprint(response, http.StatusText(statusCode))
	return err
}

func RespondWithErrorMessage(response http.ResponseWriter, request *http.Request, statusCode int, errorMessage string) error {
	return RespondWithErrorObject(response, request, statusCode, genericErrorResponse{errorMessage})
}

func RespondWithError(response http.ResponseWriter, request *http.Request, statusCode int) error {
	return RespondWithErrorMessage(response, request, statusCode, http.StatusText(statusCode))
}

func (response genericErrorResponse) String() string {
	return response.Message
}
