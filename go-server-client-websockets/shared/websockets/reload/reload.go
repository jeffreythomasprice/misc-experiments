package reload

import "time"

type ServerHelloMessage struct {
	ServerStartupTimeNumber int64 `json:"serverStartupTime"`
}

func NewServerHelloMessage() ServerHelloMessage {
	return ServerHelloMessage{
		ServerStartupTimeNumber: time.Now().UnixMilli(),
	}
}

func (message ServerHelloMessage) ServerStartupTime() time.Time {
	return time.UnixMilli(message.ServerStartupTimeNumber)
}
