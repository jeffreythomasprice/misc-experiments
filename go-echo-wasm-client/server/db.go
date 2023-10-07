package main

import (
	"database/sql"
	"errors"
	"log/slog"
	"os"
	"path"
)

type db struct {
	db *sql.DB
}

func NewDB() (*db, error) {
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

	_, err = result.Exec(`
		create table if not exists properties (
			name text not null unique,
			value text not null
		);
	`)
	if err != nil {
		return nil, err
	}

	return &db{result}, nil
}

func (db *db) Close() {
	if err := db.db.Close(); err != nil {
		slog.Error("error closing db", "err", err)
	}
}

func (db *db) GetProperty(name string) (string, error) {
	s, err := db.db.Prepare("select (value) from properties where name = ?")
	if err != nil {
		return "", err
	}
	defer func() {
		if err := s.Close(); err != nil {
			slog.Error("error cleaning up prepared statement", "err", err)
		}
	}()

	var value string
	if err := s.QueryRow(name).Scan(&value); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return "", nil
		}
		return "", err
	}
	return value, nil
}

func (db *db) SetProperty(name, value string) error {
	s, err := db.db.Prepare("insert into properties (name, value) values (?, ?) on conflict(name) do update set value=?")
	if err != nil {
		return err
	}
	defer func() {
		if err := s.Close(); err != nil {
			slog.Error("error cleaning up prepared statement", "err", err)
		}
	}()

	_, err = s.Exec(name, value, value)
	return err
}
