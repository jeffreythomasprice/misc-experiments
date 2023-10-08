package shared

import (
	"github.com/golang-jwt/jwt"
)

type JWTCustomClaims struct {
	Username string `json:"username"`
	IsAdmin  bool   `json:"isAdmin"`
}

type JWTClaims struct {
	jwt.StandardClaims
	JWTCustomClaims
}

func ParseJWTClaimsUnverified(token string) (*JWTClaims, error) {
	var result JWTClaims
	_, _, err := new(jwt.Parser).ParseUnverified(token, &result)
	return &result, err
}
