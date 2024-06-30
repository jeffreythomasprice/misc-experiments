package main

import (
	"database/sql"
	"fmt"

	"github.com/jmoiron/sqlx"
)

type User struct {
	Username string
	Password sql.NullString
}

func getUsers(db *sqlx.DB) ([]User, error) {
	results := make([]User, 0)
	err := db.Select(&results, "SELECT username FROM users")
	return results, err
}

func getUserByName(db *sqlx.DB, username string) (*User, error) {
	results := make([]User, 0)
	err := db.Select(&results, "SELECT username FROM users WHERE username = ?", username)
	if err != nil {
		return nil, fmt.Errorf("failed to get user by name: %w", err)
	}
	if len(results) == 0 {
		return nil, fmt.Errorf("no such user: %v", username)
	}
	if len(results) >= 2 {
		return nil, fmt.Errorf("duplicate users: %v", username)
	}
	return &results[0], nil
}

func createUser(db *sqlx.DB, user *User) error {
	_, err := db.Exec("INSERT INTO users (username, password) VALUES (:username, :password)", user)
	if err != nil {
		return fmt.Errorf("failed to insert user: %w", err)
	}
	return nil
}

func checkPassword(db *sqlx.DB, username, password string) (bool, error) {
	var result int
	err := db.QueryRow("SELECT COUNT(*) FROM users WHERE username = ? AND password = ?", username, password).Scan(&result)
	if err != nil {
		return false, fmt.Errorf("failed to check password: %w", err)
	}
	return result >= 1, nil
}
