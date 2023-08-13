package demo

import (
	"fmt"
	"reflect"
	"shared"
)

// TODO JEFF demo

type Message interface {
	String() string
}

/*
ClientSetName is sent by clients when they are changing what name they want to be known by
*/
type ClientSetName struct {
	Type string `json:"type" union:"-"`
	Name string `json:"name"`
}

var _ Message = (*ClientSetName)(nil)

/*
ServerInformClientsAboutNameChange is sent by the server to clients to inform them about the names of other clients
*/
type ServerInformClientsAboutNameChange struct {
	Type string `json:"type" union:"-"`
	Id   string `json:"uuid"`
	Name string `json:"name"`
}

var _ Message = (*ServerInformClientsAboutNameChange)(nil)

/*
ClientMessage is sent by clients to submit a message
*/
type ClientMessage struct {
	Type    string `json:"type" union:"-"`
	Message string `json:"message"`
}

var _ Message = (*ClientMessage)(nil)

/*
ServerInformClientsAboutMessage is sent by the server to clients to inform them about new messages
*/
type ServerInformClientsAboutMessage struct {
	Type      string `json:"type" union:"serverMessage"`
	SourceId  string `json:"id"`
	Timestamp int64  `json:"timestamp"`
	Message   string `json:"message"`
}

var _ Message = (*ServerInformClientsAboutMessage)(nil)

var MessageTaggedUnion *shared.JsonTaggedUnion[interface{}]

func init() {
	var err error
	MessageTaggedUnion, err = shared.NewJsonTaggedUnion[interface{}](
		&ClientSetName{},
		&ServerInformClientsAboutNameChange{},
		&ClientMessage{},
		&ServerInformClientsAboutMessage{},
	)
	if err != nil {
		panic(err)
	}
}

// String implements Message.
func (message *ClientSetName) String() string {
	return fmt.Sprintf("%v(%v)", reflect.TypeOf(message).Name(), message.Name)
}

// String implements Message.
func (message *ServerInformClientsAboutNameChange) String() string {
	return fmt.Sprintf("%v(%v is now %v)", reflect.TypeOf(message).Name(), message.Id, message.Name)
}

// String implements Message.
func (message *ClientMessage) String() string {
	return fmt.Sprintf("%v(%v)", reflect.TypeOf(message).Name(), message.Message)
}

// String implements Message.
func (message *ServerInformClientsAboutMessage) String() string {
	return fmt.Sprintf("%v(%v: %v sent %v)", reflect.TypeOf(message).Name(), message.Timestamp, message.SourceId, message.Message)
}
