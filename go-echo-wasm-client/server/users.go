package main

import (
	"database/sql"
	"errors"
	"fmt"
)

type UsersService struct {
	db *sql.DB

	getByNameStatement        *sql.Stmt
	validatePasswordStatement *sql.Stmt
	createStatement           *sql.Stmt
}

type User struct {
	Username string
	IsAdmin  bool
}

type CreateUser struct {
	User
	Password string
}

func NewUsersService(db *sql.DB) (*UsersService, error) {
	_, err := db.Exec(`
		create table if not exists users (
			username varchar(256) not null unique,
			password varchar(256) not null,
			isAdmin boolean
		);
	`)
	if err != nil {
		return nil, err
	}

	result := &UsersService{
		db: db,
	}

	result.getByNameStatement, err = db.Prepare("select username, isAdmin from users where username = ?")
	if err != nil {
		result.Close()
		return nil, err
	}

	result.validatePasswordStatement, err = db.Prepare("select password from users where username = ?")
	if err != nil {
		result.Close()
		return nil, err
	}

	result.createStatement, err = db.Prepare("insert into users (username, password, isAdmin) values (?, ?, ?)")
	if err != nil {
		result.Close()
		return nil, err
	}

	if err := result.createUserIfMissing(&CreateUser{
		User: User{
			Username: "admin",
			IsAdmin:  true,
		},
		Password: "admin",
	}); err != nil {
		result.Close()
		return nil, err
	}

	return result, nil
}

func (service *UsersService) Close() {
	// TODO clean up statements
}

func (service *UsersService) GetByName(username string) (*User, error) {
	var result User
	if err := service.getByNameStatement.QueryRow(username).Scan(&result.Username, &result.IsAdmin); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}
	return &result, nil
}

func (service *UsersService) ValidatePassword(username, password string) (*User, error) {
	var actualPassword string
	if err := service.validatePasswordStatement.QueryRow(username).Scan(&actualPassword); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, nil
		}
		return nil, err
	}
	if password != actualPassword {
		return nil, nil
	}
	result, err := service.GetByName(username)
	if err != nil {
		return nil, err
	}
	if result == nil {
		return nil, fmt.Errorf("validated password successfully, but then attempting to look up actual user data failed, username: %v", username)
	}
	return result, nil
}

func (service *UsersService) Create(user *CreateUser) error {
	_, err := service.createStatement.Exec(user.Username, user.Password, user.IsAdmin)
	return err
}

func (service *UsersService) createUserIfMissing(user *CreateUser) error {
	existing, err := service.GetByName(user.Username)
	if err != nil {
		return err
	}
	if existing != nil {
		return nil
	}
	return service.Create(user)
}
