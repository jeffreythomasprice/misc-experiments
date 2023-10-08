package main

import (
	"database/sql"
	"errors"
)

type PropertiesService struct {
	db *sql.DB

	getStatement *sql.Stmt
	setStatement *sql.Stmt
}

func NewPropertiesService(db *sql.DB) (*PropertiesService, error) {
	_, err := db.Exec(`
		create table if not exists properties (
			name text not null unique,
			value text not null
		);
	`)
	if err != nil {
		return nil, err
	}

	result := &PropertiesService{
		db: db,
	}

	result.getStatement, err = db.Prepare("select (value) from properties where name = ?")
	if err != nil {
		result.Close()
		return nil, err
	}

	result.setStatement, err = db.Prepare("insert into properties (name, value) values (?, ?) on conflict(name) do update set value=?")
	if err != nil {
		result.Close()
		return nil, err
	}

	return result, nil
}

func (service *PropertiesService) Close() {
	// TODO clean up statements
}

func (service *PropertiesService) Get(name string) (string, error) {
	var value string
	if err := service.getStatement.QueryRow(name).Scan(&value); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return "", nil
		}
		return "", err
	}
	return value, nil
}

func (service *PropertiesService) Set(name, value string) error {
	_, err := service.setStatement.Exec(name, value, value)
	return err
}
