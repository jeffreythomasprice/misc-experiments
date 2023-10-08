package main

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"log/slog"
	"shared"
	"time"

	"github.com/golang-jwt/jwt"
)

type JWTService struct {
	key *rsa.PrivateKey
}

func NewJWTService(props *PropertiesService) (*JWTService, error) {
	const propName = "jwt private key"

	pemStr, err := props.Get(propName)
	if err != nil {
		return nil, err
	}
	if len(pemStr) > 0 {
		slog.Debug("existing jwt key found", "pem", pemStr)
		keyBytes, _ := pem.Decode([]byte(pemStr))
		key, err := x509.ParsePKCS1PrivateKey(keyBytes.Bytes)
		if err != nil {
			return nil, err
		}
		return &JWTService{key}, nil
	}

	key, err := rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return nil, err
	}

	pemStr = string(pem.EncodeToMemory(&pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(key),
	}))
	if err := props.Set(propName, pemStr); err != nil {
		return nil, err
	}

	slog.Debug("generated new jwt key", "pem", pemStr)

	return &JWTService{key}, nil
}

func (service *JWTService) Create(claims shared.JWTCustomClaims) (string, *shared.JWTClaims, error) {
	allClaims := &shared.JWTClaims{
		StandardClaims: jwt.StandardClaims{
			IssuedAt:  time.Now().Unix(),
			ExpiresAt: time.Now().Add(time.Hour * 24).Unix(),
		},
		JWTCustomClaims: claims,
	}
	token := jwt.NewWithClaims(jwt.SigningMethodRS256, allClaims)
	result, err := token.SignedString(service.key)
	if err != nil {
		return "", nil, err
	}
	slog.Debug("issued new jwt", "jwt", result)
	return result, allClaims, nil
}

func (service *JWTService) Validate(token string) (*shared.JWTClaims, error) {
	var claims shared.JWTClaims
	_, err := jwt.ParseWithClaims(token, &claims, func(t *jwt.Token) (interface{}, error) {
		return service.key.Public(), nil
	})
	if err != nil {
		return nil, err
	}
	return &claims, nil
}
