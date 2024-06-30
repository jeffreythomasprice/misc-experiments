package main

import (
	"fmt"
	"time"

	"github.com/jmoiron/sqlx"
	"github.com/rs/zerolog/log"
)

type Migration struct {
	Name string    `db:"name"`
	Time time.Time `db:"time"`
}

func migrate(db *sqlx.DB) error {
	db.Exec(`
		CREATE TABLE IF NOT EXISTS migrations (
			name string primary key not null,
			time datetime
		)
	`)

	existingMigrations := make([]Migration, 0)
	err := db.Select(&existingMigrations, "SELECT name, time FROM migrations ORDER BY time DESC")
	if err != nil {
		return fmt.Errorf("failed to get existing migrations: %w", err)
	}
	existingMigrationsByName := make(map[string]Migration)
	for _, m := range existingMigrations {
		log.Trace().Str("name", m.Name).Time("time", m.Time).Msg("existing migration")
		existingMigrationsByName[m.Name] = m
	}

	for _, m := range []struct {
		name string
		f    func(*sqlx.DB) error
	}{
		{"create users", createUsersTable},
	} {
		log := log.With().Str("name", m.name).Logger()
		_, ok := existingMigrationsByName[m.name]
		if ok {
			log.Trace().Msg("already exists")
			continue
		}
		err = m.f(db)
		if err != nil {
			log.Error().Err(err).Msg("failed to execute")
			return fmt.Errorf("failed to insert migration record: %v", m.name)
		}
		_, err := db.Exec("INSERT INTO migrations (name, time) VALUES (?, DATE())", m.name)
		if err != nil {
			log.Error().Err(err).Msg("failed to insert record")
			return fmt.Errorf("failed to insert migration record: %v", m.name)
		}
	}
	return nil
}

func createUsersTable(db *sqlx.DB) error {
	_, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS users (
			username string PRIMARY KEY NOT NULL,
			password string NOT NULL
		)
	`)
	return err
}
