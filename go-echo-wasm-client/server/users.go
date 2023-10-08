package main

import (
	"database/sql"
	"errors"
)

type UsersService struct {
	db *sql.DB

	getByNameStatement        *sql.Stmt
	validatePasswordStatement *sql.Stmt
	createStatement           *sql.Stmt
}

type User struct {
	Username string
}

func NewUsersService(db *sql.DB) (*UsersService, error) {
	_, err := db.Exec(`
		create table if not exists users (
			username varchar(256) not null unique,
			password varchar(256) not null
		);
	`)
	if err != nil {
		return nil, err
	}

	result := &UsersService{
		db: db,
	}

	result.getByNameStatement, err = db.Prepare("select (username) from users where username = ?")
	if err != nil {
		result.Close()
		return nil, err
	}

	result.validatePasswordStatement, err = db.Prepare("select (password) from users where username = ?")
	if err != nil {
		result.Close()
		return nil, err
	}

	result.createStatement, err = db.Prepare("insert into users (username, password) values (?, ?)")
	if err != nil {
		result.Close()
		return nil, err
	}

	// create the default user if missing
	defaultUser, err := result.GetByName("admin")
	if err != nil {
		result.Close()
		return nil, err
	}
	if defaultUser == nil {
		if err := result.Create("admin", "admin"); err != nil {
			result.Close()
			return nil, err
		}
	}

	return result, nil
}

func (service *UsersService) Close() {
	// TODO clean up statements
}

func (service *UsersService) GetByName(username string) (*User, error) {
	var resultUsername string
	if err := service.getByNameStatement.QueryRow(username).Scan(&resultUsername); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}
	return &User{
		Username: resultUsername,
	}, nil
}

func (service *UsersService) ValidatePassword(username, password string) (bool, error) {
	var actualPassword string
	if err := service.validatePasswordStatement.QueryRow(username).Scan(&actualPassword); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return false, nil
		}
		return false, err
	}
	return password == actualPassword, nil
}

func (service *UsersService) Create(username, password string) error {
	_, err := service.createStatement.Exec(username, password)
	return err
}
