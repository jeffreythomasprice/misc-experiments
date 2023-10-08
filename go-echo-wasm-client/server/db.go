package main

import (
	"database/sql"
	"log/slog"
	"os"
	"path"
)

func openDatabase() (*sql.DB, error) {
	exePath, err := os.Executable()
	if err != nil {
		return nil, err
	}
	dbPath := path.Join(path.Dir(exePath), "db")
	slog.Debug("newDB", "exePath", exePath, "dbPath", dbPath)

	result, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, err
	}

	return result, nil
}
