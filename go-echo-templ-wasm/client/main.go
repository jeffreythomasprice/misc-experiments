package main

import (
	"bytes"
	"client/swap"
	"encoding/json"
	"io"
	"net/http"
	"shared"
	"syscall/js"

	"github.com/rs/zerolog/log"
)

func main() {
	shared.InitLogging(false)

	var usernameInput, passwordInput js.Value
	shared.Must0(swap.Swap(
		index(),
		map[string]any{
			"submit": js.FuncOf(func(this js.Value, args []js.Value) any {
				e := args[0]
				e.Call("preventDefault")
				log.Debug().
					Str("username", usernameInput.Get("value").String()).
					Str("password", passwordInput.Get("value").String()).
					Msg("TODO actually submit")
				request := &shared.LoginRequest{
					Username: usernameInput.Get("value").String(),
					Password: passwordInput.Get("value").String(),
				}
				go func() {
					var responseBody shared.Result[shared.LoginResponse]
					response := shared.Must(makeJsonRequest(
						http.MethodPost,
						"/login",
						request,
						&responseBody,
					))
					log.Debug().Int("status", response.StatusCode).Interface("response", response).Msg("TODO response")
				}()
				return nil
			}),
		},
		map[string]*js.Value{
			"username": &usernameInput,
			"password": &passwordInput,
		},
		"body",
		swap.InnerHTML,
	))

	select {}
}

func makeJsonRequest(method, url string, requestBody interface{}, responseBody interface{}) (*http.Response, error) {
	log := log.With().Str("method", method).Str("url", url).Logger()
	var requestBodyReader io.Reader
	if requestBody != nil {
		requestBodyBytes, err := json.Marshal(requestBody)
		if err != nil {
			log.Err(err).Msg("error marshalling request body")
			return nil, err
		}
		requestBodyReader = bytes.NewBuffer(requestBodyBytes)
	}
	request, err := http.NewRequest(method, url, requestBodyReader)
	if err != nil {
		log.Err(err).Msg("error making request")
		return nil, err
	}
	request.Header.Add("content-type", "application/json")
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		log.Err(err).Msg("error making request")
		return nil, err
	}
	log.Trace().Int("status", response.StatusCode).Msg("response")
	if responseBody != nil {
		responseBodyBytes, err := io.ReadAll(response.Body)
		if err != nil {
			log.Err(err).Msg("error reading response body")
			return nil, err
		}
		if err := json.Unmarshal(responseBodyBytes, responseBody); err != nil {
			log.Err(err).Msg("error unmarshalling reponse body")
			return nil, err
		}
	}
	return response, nil
}
