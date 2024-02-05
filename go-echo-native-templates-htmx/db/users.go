package db

import "github.com/jmoiron/sqlx"

type User struct {
	Username string `db:"username"`
	IsAdmin  bool   `db:"isAdmin"`
}

type CreateUser struct {
	Username string
	Password string
	IsAdmin  bool
}

func (service *Service) createUserTable() error {
	_, err := service.db.Exec(`
		CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			username TEXT NOT NULL UNIQUE,
			password TEXT NOT NULL,
			isAdmin BOOLEAN NOT NULL
		);
	`)
	return err
}

func (service *Service) seedUserData(tx *sqlx.Tx) error {
	for _, user := range []CreateUser{
		{
			Username: "admin",
			Password: "admin",
			IsAdmin:  true,
		}, {
			Username: "user",
			Password: "password",
		},
	} {
		existing, err := service.GetUserByUsername(tx, user.Username)
		if err != nil {
			return err
		}
		if existing == nil {
			err = service.CreateUser(tx, user)
			if err != nil {
				return err
			}
		}
	}
	return nil
}

func (service *Service) GetUserByUsername(q sqlx.Queryer, username string) (*User, error) {
	return getSingle[User](
		service.q(q),
		"SELECT username, isAdmin FROM users WHERE username = ?",
		username,
	)
}

func (service *Service) CheckPassword(q sqlx.Queryer, username, password string) (*User, error) {
	return getSingle[User](
		service.q(q),
		"SELECT username, isAdmin FROM users WHERE username = ? AND password = ?",
		username,
		password,
	)
}

func (service *Service) CreateUser(e sqlx.Execer, user CreateUser) error {
	log := service.log.With().Str("username", user.Username).Logger()
	log.Info().Msg("creating user")
	_, err := service.e(e).Exec(
		"INSERT INTO users (username, password, isAdmin) VALUES (?, ?, ?)",
		user.Username,
		user.Password,
		user.IsAdmin,
	)
	return err
}
