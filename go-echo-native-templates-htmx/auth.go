package main

import (
	"errors"
	"experiment/db"
	"fmt"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog"
)

var errUnauthorized = errors.New("Unauthorized")

func createAuthToken(log *zerolog.Logger, c echo.Context, user *db.User) error {
	exp := time.Now().Add(30 * time.Second)
	token, err := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"exp":      exp.Unix(),
		"username": user.Username,
	}).
		SignedString([]byte("TODO secret key here"))
	if err != nil {
		log.Error().Err(err).Msg("failed to create auth token")
		return err
	}
	log.Trace().Str("token", token).Msg("created auth token")
	c.SetCookie(&http.Cookie{
		Name:    "token",
		Value:   token,
		Expires: exp,
	})
	return nil
}

func checkAuthToken(c echo.Context, service *db.Service) (*db.User, error) {
	cookie, err := c.Cookie("token")
	if errors.Is(err, echo.ErrCookieNotFound) || errors.Is(err, http.ErrNoCookie) || (err == nil && cookie == nil) {
		return nil, errUnauthorized
	}
	if err != nil {
		return nil, err
	}
	token, err := jwt.Parse(cookie.Value, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header)
		}
		return []byte("TODO secret key here"), nil
	})
	if errors.Is(err, jwt.ErrTokenExpired) {
		return nil, errUnauthorized
	}
	if err != nil {
		return nil, err
	}
	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		return nil, fmt.Errorf("unexpected claims type: %v", token.Claims)
	}
	username, ok := claims["username"]
	if !ok {
		return nil, fmt.Errorf("username missing from token: %v", claims)
	}
	usernameStr, ok := username.(string)
	if !ok {
		return nil, fmt.Errorf("username present in token, but not a string: %v", claims)
	}
	return service.GetUserByUsername(nil, usernameStr)
}
