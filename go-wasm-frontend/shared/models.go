package shared

type WebsocketLoginRequest struct {
	Name string
}

type WebsocketLoginResponse struct {
	ID string
}

type WebsocketClientToServerMessageLogin struct {
	ID string
}

type WebsocketClientToServerMessageSend struct {
	Message string
}

type WebsocketClientToServerMessage struct {
	Type  string
	Login *WebsocketClientToServerMessageLogin `json:",omitempty"`
	Send  *WebsocketClientToServerMessageSend  `json:",omitempty"`
}

type WebsocketServerToClientMessageSend struct {
	SenderID string
	Message  string
}

type WebsocketServerToClientMessage struct {
	Type string
	Send *WebsocketServerToClientMessageSend `json:",omitempty"`
}

const WebsocketClientToServerMessageTypeLogin = "Login"
const WebsocketClientToServerMessageTypeSend = "Send"
const WebsocketServerToClientMessageTypeSend = "Send"
