package main

import (
	"github.com/gin-gonic/gin"
	"github.com/rs/zerolog/log"
)

func initGin() *gin.Engine {
	gin.SetMode(gin.ReleaseMode)

	result := gin.New()

	result.Use(gin.LoggerWithFormatter(func(params gin.LogFormatterParams) string {
		log.Debug().
			Int("status code", params.StatusCode).
			Str("latency", params.Latency.String()).
			Str("client IP", params.ClientIP).
			Str("method", params.Method).
			Str("path", params.Path).
			Msg("gin")
		return ""
	}))

	result.Use(gin.Recovery())

	return result
}

func runGin(g *gin.Engine, addr ...string) {
	log.Info().Strs("addr", addr).Msg("starting server")
	if err := g.Run(addr...); err != nil {
		log.Fatal().Err(err).Msg("server error")
	}
}
