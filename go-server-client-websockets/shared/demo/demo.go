package demo

import (
	"encoding/json"
	"fmt"
	"time"
)

// TODO JEFF demo

/*
ClientSetName is sent by clients when they are changing what name they want to be known by
*/
type ClientSetName struct {
	Name string `json:"name"`
}

/*
ServerInformClientsAboutNameChange is sent by the server to clients to inform them about the names of other clients
*/
type ServerInformClientsAboutNameChange struct {
	Id   string `json:"uuid"`
	Name string `json:"name"`
}

/*
ClientMessage is sent by clients to submit a message
*/
type ClientMessage struct {
	Message string `json:"message"`
}

/*
ServerInformClientsAboutMessage is sent by the server to clients to inform them about new messages
*/
type ServerInformClientsAboutMessage struct {
	SourceId  string `json:"id"`
	Timestamp int64  `json:"timestamp"`
	Message   string `json:"message"`
}

type MessageWrapper struct {
	Type       string          `json:"type"`
	RawMessage json.RawMessage `json:"message"`
	Message    interface{}     `json:"-"`
}

var _ json.Unmarshaler = (*MessageWrapper)(nil)
var _ json.Marshaler = (*MessageWrapper)(nil)

const (
	MessageTypeClientSetName                      = "setName"
	MessageTypeServerInformClientsAboutNameChange = "informAboutNameChange"
	MessageTypeClientMessage                      = "clientMessage"
	MessageTypeServerInformClientsAboutMessage    = "informAboutMessage"
)

func NewClientSetName(name string) *MessageWrapper {
	return &MessageWrapper{
		Type: MessageTypeClientSetName,
		Message: &ClientSetName{
			Name: name,
		},
	}
}

func NewServerInformClientsAboutNameChange(id string, name string) *MessageWrapper {
	return &MessageWrapper{
		Type: MessageTypeServerInformClientsAboutNameChange,
		Message: &ServerInformClientsAboutNameChange{
			Id:   id,
			Name: name,
		},
	}
}

func NewClientMessage(message string) *MessageWrapper {
	return &MessageWrapper{
		Type: MessageTypeClientMessage,
		Message: &ClientMessage{
			Message: message,
		},
	}
}

func NewServerInformClientsAboutMessage(sourceId string, timestamp time.Time, message string) *MessageWrapper {
	return &MessageWrapper{
		Type: MessageTypeServerInformClientsAboutMessage,
		Message: &ServerInformClientsAboutMessage{
			SourceId:  sourceId,
			Timestamp: timestamp.UnixMilli(),
			Message:   message,
		},
	}
}

// UnmarshalJSON implements json.Unmarshaler.
func (message *MessageWrapper) UnmarshalJSON(b []byte) error {
	type t MessageWrapper
	if err := json.Unmarshal(b, (*t)(message)); err != nil {
		return err
	}
	switch message.Type {
	case MessageTypeClientSetName:
		message.Message = &ClientSetName{}
	case MessageTypeServerInformClientsAboutNameChange:
		message.Message = &ServerInformClientsAboutNameChange{}
	case MessageTypeClientMessage:
		message.Message = &ClientMessage{}
	case MessageTypeServerInformClientsAboutMessage:
		message.Message = &ServerInformClientsAboutMessage{}
	default:
		return fmt.Errorf("unrecognized message type: %v", message.Type)
	}
	if err := json.Unmarshal(message.RawMessage, message.Message); err != nil {
		return err
	}
	return nil
}

// MarshalJSON implements json.Marshaler.
func (message *MessageWrapper) MarshalJSON() ([]byte, error) {
	type t MessageWrapper
	b, err := json.Marshal(message.Message)
	if err != nil {
		return nil, err
	}
	message.RawMessage = b
	return json.Marshal((*t)(message))
}
