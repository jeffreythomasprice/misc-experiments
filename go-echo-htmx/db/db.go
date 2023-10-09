package db

import (
	"database/sql"
	"fmt"
	"log/slog"
	"path"
)

type DBService struct {
	db *sql.DB
}

func NewService(getTempDir func() (string, error)) (*DBService, error) {
	tmpDir, err := getTempDir()
	if err != nil {
		return nil, err
	}
	dbPath := path.Join(tmpDir, "db")

	result, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	if err := applySchema(result); err != nil {
		return nil, fmt.Errorf("failed to apply schema: %w", err)
	}

	return &DBService{result}, nil
}

func (service *DBService) Close() {
	if err := service.db.Close(); err != nil {
		slog.Error("error closing database", "err", err)
	}
}

func applySchema(db *sql.DB) error {
	// users

	// create the table
	if _, err := db.Exec(
		`create table if not exists users (
			username varchar(256) primary key unique not null,
			password varchar(256) not null
		)`,
	); err != nil {
		return fmt.Errorf("failed to create users table: %w", err)
	}

	// initial user data
	if _, err := db.Exec(
		`insert or ignore into users (username, password) values ("admin", "admin")`,
	); err != nil {
		return fmt.Errorf("failed to create initial user: %w", err)
	}

	return nil
}
