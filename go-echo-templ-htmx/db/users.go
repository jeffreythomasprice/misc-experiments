package db

import (
	"errors"
	"time"

	"gorm.io/gorm"
)

type User struct {
	// no, because soft delete isn't appropriate here
	// gorm.Model

	CreatedAt time.Time
	UpdatedAt time.Time

	Username string `gorm:"unique"`
	Password string
}

var ErrInvalidCredentials = errors.New("invalid credentials")

func (db *DB) CreateUser(user *User) error {
	result := db.db.Create(user)
	return result.Error
}

func (db *DB) FindUserByName(username string) (*User, error) {
	var user User
	result := db.db.First(&user, "username = ?", username)
	return &user, result.Error
}

func (db *DB) CheckCredentials(username, password string) (*User, error) {
	var user User
	result := db.db.First(&user, "username = ?", username)
	if result.Error != nil {
		if errors.Is(result.Error, gorm.ErrRecordNotFound) {
			return nil, ErrInvalidCredentials
		}
		return nil, result.Error
	}
	// TODO hashing
	if user.Password != password {
		return nil, ErrInvalidCredentials
	}
	// don't leak passwords
	user.Password = ""
	return &user, nil
}
