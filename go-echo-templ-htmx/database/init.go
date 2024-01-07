package database

import "gorm.io/gorm"

func Init(db *gorm.DB) error {
	if err := db.AutoMigrate(&Message{}); err != nil {
		return err
	}
	return nil
}
