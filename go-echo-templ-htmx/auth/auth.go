package auth

import (
	"net/http"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/labstack/echo/v4"
	"github.com/rs/zerolog"
	"github.com/ziflex/lecho/v3"
)

type NewJwtRequest struct {
	Username string
}

type Claims struct {
	jwt.RegisteredClaims
	Username string `json:"username"`
}

const cookieName = "auth"
const contextName = "auth"

// TODO real secret
var secret = []byte("secret")

func RequireAuth(handleFailure func(c echo.Context, err error) error) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			log := getLoggerFromEcho(c).With().Str("cookieName", cookieName).Logger()

			auth, err := c.Cookie(cookieName)
			if auth == nil || err != nil {
				log.Warn().Err(err).Msg("failed to get auth cookie")
				return handleFailure(c, err)
			}
			log.Trace().Str("value", auth.Value).Msg("found auth cookie")

			claims, err := validateToken(auth.Value)
			if err != nil {
				log.Warn().Err(err).Msg("failed to validate auth token")
				return handleFailure(c, err)
			}
			log.Trace().
				Str("username", claims.Username).
				Time("expiresAt", claims.ExpiresAt.Time).
				Msg("found valid claims")
			c.Set(contextName, claims)

			return next(c)
		}
	}
}

func Get(c echo.Context) *Claims {
	result, ok := c.Get(contextName).(*Claims)
	if !ok {
		return nil
	}
	return result
}

func NewCookie(c echo.Context, request *NewJwtRequest) error {
	log := getLoggerFromEcho(c).With().Str("cookieName", cookieName).Logger()
	token, claims, err := newToken(request)
	if err != nil {
		return err
	}
	cookie := &http.Cookie{
		Name:    cookieName,
		Value:   token,
		Expires: claims.ExpiresAt.Time,
	}
	log.Debug().Stringer("cookie", cookie).Msg("added auth cookie")
	c.SetCookie(cookie)
	return nil
}

func ClearCookie(c echo.Context) {
	log := getLoggerFromEcho(c).With().Str("cookieName", cookieName).Logger()
	log.Debug().Msg("clearing auth cookie")
	c.SetCookie(&http.Cookie{
		Name:    cookieName,
		Expires: time.Now(),
	})
}

func newToken(request *NewJwtRequest) (string, *Claims, error) {
	claims := Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: &jwt.NumericDate{
				Time: time.Now().Add(time.Hour * 24),
			},
		},
		Username: request.Username,
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS384, claims)
	result, err := token.SignedString(secret)
	return result, &claims, err
}

func validateToken(token string) (*Claims, error) {
	var claims Claims
	_, err := jwt.ParseWithClaims(
		token,
		&claims,
		func(t *jwt.Token) (interface{}, error) {
			return secret, nil
		},
	)
	if err != nil {
		return nil, err
	}
	return &claims, nil
}

func getLoggerFromEcho(c echo.Context) *zerolog.Logger {
	return lecho.Ctx(c.Request().Context())
}
