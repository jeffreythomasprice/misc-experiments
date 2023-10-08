package shared

import (
	"fmt"
	"net/http"
)

type ErrorResponse struct {
	Message string `json:"message"`
}

type CheckTokenResponse struct {
	Token string `json:"token"`
}

type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type LoginResponse struct {
	Token string `json:"token"`
}

type HTTPResponseError struct {
	Response     *http.Response
	ResponseBody ErrorResponse
}

var _ error = &HTTPResponseError{}

// Error implements error.
func (e *HTTPResponseError) Error() string {
	return fmt.Sprintf("statusCode=%v, message=%v", e.Response.StatusCode, e.ResponseBody.Message)
}

func CheckToken() (*CheckTokenResponse, error) {
	return MakeJsonRequest[CheckTokenResponse](http.MethodGet, "/checkToken", nil)
}

func Login(request *LoginRequest) (*LoginResponse, error) {
	return MakeJsonRequest[LoginResponse](http.MethodPost, "/login", request)
}

func Logout() error {
	return MakeJsonRequestNoResponse(http.MethodPost, "/logout", nil)
}
