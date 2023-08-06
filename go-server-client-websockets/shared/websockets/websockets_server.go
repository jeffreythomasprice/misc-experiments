//go:build !(js && wasm)

package websockets

import (
	"log/slog"
	"net/http"
	"sync"

	"github.com/olahol/melody"
)

type websocketServerImpl struct {
	incomingMutex sync.Mutex
	incoming      chan WebsocketConnection
}

var _ WebsocketServer = (*websocketServerImpl)(nil)

type websocketServerConnectionImpl struct {
	session       *melody.Session
	incomingMutex sync.Mutex
	incoming      chan []byte
}

var _ WebsocketConnection = (*websocketServerConnectionImpl)(nil)

const sessionConnectionKey = "connection"

func NewWebsocketServerHandlerFunc() (WebsocketServer, http.HandlerFunc) {
	m := melody.New()

	server := &websocketServerImpl{
		incoming: make(chan WebsocketConnection),
	}

	m.HandleConnect(func(s *melody.Session) {
		server.HandleConnect(s)
	})

	m.HandleDisconnect(func(s *melody.Session) {
		server.HandleDisconnect(s)
	})

	m.HandleMessage(func(s *melody.Session, b []byte) {
		server.HandleMessage(s, b)
	})

	m.HandleMessageBinary(func(s *melody.Session, b []byte) {
		server.HandleMessage(s, b)
	})

	return server, func(w http.ResponseWriter, r *http.Request) {
		m.HandleRequest(w, r)
	}
}

func (server *websocketServerImpl) HandleConnect(session *melody.Session) {
	slog.Debug(
		"client connected",
		"remote addr", session.RemoteAddr().String(),
	)

	connection := &websocketServerConnectionImpl{
		session:  session,
		incoming: make(chan []byte),
	}
	session.Set(sessionConnectionKey, connection)

	server.incomingMutex.Lock()
	defer server.incomingMutex.Unlock()
	server.incoming <- connection
}

func (server *websocketServerImpl) HandleDisconnect(session *melody.Session) {
	slog.Debug(
		"client disconnected",
		"remote addr", session.RemoteAddr().String(),
	)
	connection := session.MustGet(sessionConnectionKey).(*websocketServerConnectionImpl)
	connection.HandleDisconnect()
}

func (server *websocketServerImpl) HandleMessage(session *melody.Session, b []byte) {
	slog.Debug(
		"received message",
		"remote addr", session.RemoteAddr().String(),
		"length", len(b),
	)
	connection := session.MustGet(sessionConnectionKey).(*websocketServerConnectionImpl)
	connection.HandleMessage(b)
}

// Close inplements WebsocketServer.
func (server *websocketServerImpl) Close() error {
	server.incomingMutex.Lock()
	defer server.incomingMutex.Unlock()
	close(server.incoming)
	server.incoming = nil
	return nil
}

// Incoming implements WebsocketServer.
func (server *websocketServerImpl) Incoming() <-chan WebsocketConnection {
	return server.incoming
}

func (connection *websocketServerConnectionImpl) HandleDisconnect() {
	connection.incomingMutex.Lock()
	defer connection.incomingMutex.Unlock()
	close(connection.incoming)
	connection.incoming = nil
}

func (connection *websocketServerConnectionImpl) HandleMessage(b []byte) {
	connection.incomingMutex.Lock()
	defer connection.incomingMutex.Unlock()
	connection.incoming <- b
}

// Incoming implements WebsocketConnection.
func (connection *websocketServerConnectionImpl) Incoming() <-chan []byte {
	return connection.incoming
}

// ReturnIncoming implements WebsocketConnection.
func (connection *websocketServerConnectionImpl) ReturnIncoming(b []byte) {
	// noop, the websocket library is handling these slices and we can't pool them
}

// Send implements WebsocketConnection.
func (connection *websocketServerConnectionImpl) SendText(s string) error {
	if connection.session.IsClosed() {
		return ErrClosed
	}
	return connection.session.Write([]byte(s))
}

// Send implements WebsocketConnection.
func (connection *websocketServerConnectionImpl) SendBinary(b []byte) error {
	if connection.session.IsClosed() {
		return ErrClosed
	}
	return connection.session.WriteBinary(b)
}
