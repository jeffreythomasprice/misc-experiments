package db

import (
	"errors"
	"fmt"
)

type User struct {
	Username string
}

var ErrBadPassword = errors.New("passwords don't match")

type UsersService interface {
	GetUserAndValidatePassword(username, password string) (*User, error)
}

var _ UsersService = (*DBService)(nil)

// GetUserAndValidatePassword implements UsersService.
// returns the user if it exists, and it's password matches
// returns nil, sql.ErrNoRows if there is no user with that name
// returns nil, ErrBadPassword if the user exists, but has a different password
func (service *DBService) GetUserAndValidatePassword(username string, password string) (*User, error) {
	var resultUsername, resultPassword string
	if err := service.db.
		QueryRow(`select username, password from users where username = ?`, username).
		Scan(&resultUsername, &resultPassword); err != nil {
		return nil, fmt.Errorf("error finding user to validate password: %w", err)
	}
	if resultPassword != password {
		return nil, ErrBadPassword
	}
	return &User{
		Username: resultUsername,
	}, nil
}
