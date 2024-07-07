package db

import (
	"errors"
	"fmt"
	"time"

	"github.com/rs/zerolog/log"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

type DB struct {
	db *gorm.DB
}

func New(f func() (*gorm.DB, error)) (*DB, error) {
	db, err := f()
	if err != nil {
		return nil, fmt.Errorf("error opening db: %w", err)
	}

	db.Logger = logger.New(&log.Logger, logger.Config{
		Colorful:             true,
		ParameterizedQueries: true,
		LogLevel:             logger.Info,
		SlowThreshold:        time.Second * 2,
	})

	result := &DB{db: db}

	// table
	if err := db.AutoMigrate(&User{}); err != nil {
		return nil, fmt.Errorf("error migrating users table: %w", err)
	}

	// default user
	_, err = result.FindUserByName("admin")
	if errors.Is(err, gorm.ErrRecordNotFound) {
		if err := result.CreateUser(&User{
			Username: "admin",
			Password: "admin",
		}); err != nil {
			return nil, fmt.Errorf("error creating default admin user: %w", err)
		}
	} else if err != nil {
		return nil, fmt.Errorf("error creating default admin user, failed to find user: %w", err)
	}

	log.Info().Msg("db migration complete")
	return result, nil
}
