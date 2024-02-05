package db

import (
	"context"
	"database/sql"
	"errors"
	"fmt"

	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
	"github.com/rs/zerolog"
)

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

	result := &Service{
		log: log,
		db:  db,
	}

	if err := result.createUserTable(); err != nil {
		return nil, err
	}

	if err := result.tx(func(tx *sqlx.Tx) error {
		return result.seedUserData(tx)
	}); err != nil {
		return nil, err
	}

	return &Service{
		log: log,
		db:  db,
	}, nil
}

func (service *Service) tx(f func(*sqlx.Tx) error) error {
	tx, err := service.db.Beginx()
	if err != nil {
		return fmt.Errorf("error beginning new transaction: %w", err)
	}
	if err := f(tx); err != nil {
		if err := tx.Rollback(); err != nil {
			service.log.Error().Err(err).Msg("while rolling back a transaction because of an error, another error occurred")
		}
		return fmt.Errorf("error performing work inside a transacction: %w", err)
	}
	if err := tx.Commit(); err != nil {
		return fmt.Errorf("error committing transaction: %w", err)
	}
	return nil
}

func (service *Service) q(q sqlx.Queryer) sqlx.Queryer {
	if q == nil {
		return service.db
	}
	return q
}

func (service *Service) e(e sqlx.Execer) sqlx.Execer {
	if e == nil {
		return service.db
	}
	return e
}

func getSingle[T any](q sqlx.Queryer, query string, args ...any) (*T, error) {
	var result T
	err := sqlx.Get(q, &result, query, args...)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &result, nil
}
