package db

import (
	"context"
	"fmt"

	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
	"github.com/rs/zerolog"
)

type User struct {
	Username string `db:"username"`
	IsAdmin  string `db:"isAdmin"`
}

type Service struct {
	log zerolog.Logger
	db  *sqlx.DB
}

func NewService(ctx context.Context) (*Service, error) {
	log := zerolog.Ctx(ctx).With().Str("service", "db").Logger()

	db, err := sqlx.ConnectContext(ctx, "sqlite3", "local.db")
	if err != nil {
		return nil, err
	}

	if _, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			username TEXT NOT NULL,
			password TEXT NOT NULL,
			isAdmin BOOLEAN NOT NULL
		);
	`); err != nil {
		return nil, err
	}

	// TODO transaction helper, takes a function that returns error?
	tx, err := db.Beginx()
	if err != nil {
		return nil, err
	} else {
		rollback := func() {
			if err := tx.Rollback(); err != nil {
				log.Error().Err(err).Msg("while handling a previous error, an error occurred rolling back the transaction")
			}
		}
		var count int64
		err := tx.Get(&count, "SELECT count(*) FROM users WHERE username = ?", "admin")
		if err != nil {
			rollback()
			return nil, err
		}
		if count == 0 {
			log.Info().Msg("creating admin user")
			_, err = tx.Exec("INSERT INTO users (username, password, isAdmin) VALUES (?, ?, ?)", "admin", "admin", true)
			if err != nil {
				rollback()
				return nil, err
			}
		}
		if err := tx.Commit(); err != nil {
			return nil, err
		}
	}

	return &Service{
		log: log,
		db:  db,
	}, nil
}

func (service *Service) CheckPassword(username, password string) (bool, error) {
	log := service.log.With().Str("username", username).Logger()
	var count int64
	err := service.db.Get(&count, "SELECT count(*) FROM users WHERE username = ? AND password = ?", username, password)
	if err != nil {
		log.Trace().Err(err).Msg("failed to check password")
		return false, err
	}
	if count > 1 {
		log.Trace().Int64("count", count).Msg("too many affected rows for check password")
		return false, fmt.Errorf("expected a single user but got duplicates, username = %s", username)
	}
	if count == 1 {
		log.Trace().Msg("password matched")
		return true, nil
	}
	log.Trace().Msg("password did not match")
	return false, nil
}
