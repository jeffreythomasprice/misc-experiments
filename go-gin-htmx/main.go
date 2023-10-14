package main

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func main() {
	initLogger()

	g := initGin()

	page := pageRenderer(g, &pageRendererOptions{
		liveReload: true,
	})

	g.GET("/", func(ctx *gin.Context) {
		page(ctx, newTemplateStringer(assetsEmbedTemplates, "clicks.html"), nil)
	})

	clicks(g)

	runGin(g, "127.0.0.1:8000")
}

func clicks(r gin.IRouter) {
	clicks := 0

	r.GET("/click", func(ctx *gin.Context) {
		ctx.HTML(http.StatusOK, "clickResults", clicks)
	})

	r.POST("/click", func(ctx *gin.Context) {
		clicks++
		ctx.HTML(http.StatusOK, "clickResults", clicks)
	})
}
