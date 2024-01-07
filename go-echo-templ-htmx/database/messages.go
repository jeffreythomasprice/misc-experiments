package database

import (
	"fmt"
	"time"

	"gorm.io/gorm"
)

type Message struct {
	ID        uint `gorm:"primarykey"`
	CreatedAt time.Time
	UpdatedAt time.Time
	Message   string
}

func (m Message) String() string {
	return fmt.Sprintf("Message(id=%v, message=\"%v\")", m.ID, m.Message)
}

func ListMessages(db *gorm.DB) ([]*Message, error) {
	var results []*Message
	err := db.Find(&results).Error
	if err != nil {
		return nil, err
	}
	return results, nil
}

func GetMessage(db *gorm.DB, id uint) (*Message, error) {
	var result Message
	if err := db.First(&result, id).Error; err != nil {
		return nil, err
	}
	return &result, nil
}

func CreateMessage(db *gorm.DB, message *Message) (*Message, error) {
	err := db.Create(message).Error
	if err != nil {
		return nil, err
	}
	return message, nil
}

func UpdateMessage(db *gorm.DB, message *Message) (*Message, error) {
	err := db.Updates(message).Error
	if err != nil {
		return nil, err
	}
	return message, nil
}

func DeleteMessage(db *gorm.DB, id uint) error {
	return db.Delete(&Message{}, id).Error
}
