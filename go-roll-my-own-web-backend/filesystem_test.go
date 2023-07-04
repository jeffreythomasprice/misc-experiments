package main

import (
	"io/fs"
	"testing"

	"github.com/psanford/memfs"
)

func TestGetAllFilesRecursively(t *testing.T) {
	root := memfs.New()
	root.MkdirAll("schemas/foo/bar", fs.ModeDir)
	root.MkdirAll("schemas/baz", fs.ModeDir)
	root.WriteFile("schemas/foo/bar/1.json", []byte("test 1"), 0)
	root.WriteFile("schemas/foo/bar/2.json", []byte("test 2"), 0)
	root.WriteFile("schemas/foo/3.json", []byte("test 3"), 0)
	root.WriteFile("schemas/bar/4.json", []byte("test 4"), 0)

	files, err := GetAllFilesRecursively(root, ".")
	t.NoEr

	// TODO use testing lib? https://github.com/matryer/is
}
