package main

import (
	"embed"
	_ "embed"
	"io/fs"
	"net/http"
)

//go:embed static
var staticDir embed.FS

//go:embed generated
var generatedDir embed.FS

func main() {
	{
		subFs, err := fs.Sub(staticDir, "static")
		if err != nil {
			panic(err)
		}
		http.Handle("/", http.FileServerFS(subFs))
	}

	{
		subFs, err := fs.Sub(generatedDir, "generated")
		if err != nil {
			panic(err)
		}
		http.Handle("/generated/", http.StripPrefix("/generated/", http.FileServerFS(subFs)))
	}

	{
		err := http.ListenAndServe("127.0.0.1:8000", nil)
		if err != nil {
			panic(err)
		}
	}
}
