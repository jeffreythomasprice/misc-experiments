package auth

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

var ErrUnauthorized = errors.New("Unauthorized")

var cookieName string = "token"
var jwtSecret []byte = []byte("super secret password")

func CreateToken(log *zerolog.Logger, c echo.Context, user *db.User) error {
	exp := time.Now().Add(60 * time.Minute)
	token, err := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"exp":      exp.Unix(),
		"username": user.Username,
	}).
		SignedString(jwtSecret)
	if err != nil {
		log.Error().Err(err).Msg("failed to create auth token")
		return err
	}
	log.Trace().Str("token", token).Msg("created auth token")
	c.SetCookie(&http.Cookie{
		Name:    cookieName,
		Value:   token,
		Expires: exp,
	})
	return nil
}

func CheckToken(log *zerolog.Logger, c echo.Context, service *db.Service) (*zerolog.Logger, *db.User, error) {
	if log == nil {
		log = zerolog.Ctx(c.Request().Context())
	}
	cookie, err := c.Cookie("token")
	if errors.Is(err, echo.ErrCookieNotFound) || errors.Is(err, http.ErrNoCookie) || (err == nil && cookie == nil) {
		return log, nil, ErrUnauthorized
	}
	if err != nil {
		return log, nil, err
	}
	token, err := jwt.Parse(cookie.Value, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header)
		}
		return jwtSecret, nil
	})
	if errors.Is(err, jwt.ErrTokenExpired) {
		return log, nil, ErrUnauthorized
	}
	if err != nil {
		return log, nil, err
	}
	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		return log, nil, fmt.Errorf("unexpected claims type: %v", token.Claims)
	}
	username, ok := claims["username"]
	if !ok {
		return log, nil, fmt.Errorf("username missing from token: %v", claims)
	}
	usernameStr, ok := username.(string)
	if !ok {
		return log, nil, fmt.Errorf("username present in token, but not a string: %v", claims)
	}
	user, err := service.GetUserByUsername(nil, usernameStr)
	if user != nil {
		updatedLog := log.With().Str("username", user.Username).Logger()
		log = &updatedLog
	}
	return log, user, err
}

func ClearToken(c echo.Context) {
	c.SetCookie(&http.Cookie{
		Name:    cookieName,
		Expires: time.Now(),
	})
}
