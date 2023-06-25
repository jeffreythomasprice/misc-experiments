package main

import (
	"io/fs"
	"path"
)

type HasPath interface {
	Path() string
}

type Openable interface {
	Open() (fs.File, error)
}

type DirEntryInFS struct {
	fs.DirEntry
	files fs.FS
	root  string
}

var _ HasPath = DirEntryInFS{}
var _ Openable = DirEntryInFS{}

func (e DirEntryInFS) Path() string {
	return path.Join(e.root, e.Name())
}

func (e DirEntryInFS) Open() (fs.File, error) {
	return e.files.Open(e.Path())
}

func getAllFilesRecursively(files fs.ReadDirFS, root string) ([]DirEntryInFS, error) {
	dirEnts, err := files.ReadDir(root)
	if err != nil {
		return nil, err
	}
	results := []DirEntryInFS{}
	for _, e := range dirEnts {
		if e.IsDir() {
			children, err := getAllFilesRecursively(files, path.Join(root, e.Name()))
			if err != nil {
				return nil, err
			}
			results = append(results, children...)
		} else {
			results = append(results, DirEntryInFS{e, files, root})
		}
	}
	return results, nil
}
