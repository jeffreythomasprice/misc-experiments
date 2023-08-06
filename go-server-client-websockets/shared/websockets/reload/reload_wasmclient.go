//go:build js && wasm

package reload

import (
	"errors"
	"log/slog"
	"shared"
	"shared/websockets"
	"syscall/js"
	"time"
)

func StartAutoReloadClient(addr string) {
	// always set to now
	// if the client restarts it'll call this function again, getting us a new time
	clientStartTime := time.Now()

	// will be set to the time the server reports when we connect
	// until then it's the zero time
	var serverStartTime time.Time

	// the main body of the connection
	// each run through this function represents one attempt to connect and process messages until the connection dies
	doIt := func() error {
		connection, err := websockets.NewWebsocketConnection(addr)
		if err != nil {
			return err
		}

		jsonConnection := websockets.NewJsonWebsocketConnection[ServerHelloMessage](connection)

		// loop while the connection is alive
		for message := range jsonConnection.Incoming() {
			// TODO should be trace
			slog.Debug(
				"incoming message",
				"time", message.ServerStartupTime().Format(shared.ISO8601_MILLIS_FORMAT),
			)

			shouldRestart := false
			if serverStartTime.IsZero() {
				// this must be the first such message we've received
				// if enough time has passed since we first started checking assume we're stale and need restart anyway
				if time.Now().Sub(clientStartTime) > time.Second*10 {
					slog.Info("first server hello received after client has already been up for a while, restarting")
					shouldRestart = true
				}
			} else {
				// we have a server time already
				// if the new time is different it means the server must have restarted
				// we should restart too, in case the client code is different
				if !message.ServerStartupTime().Equal(serverStartTime) {
					slog.Info("server hello time doesn't match, restarting")
					shouldRestart = true
				}
			}

			if shouldRestart {
				slog.Debug("restarting...")
				js.Global().Get("location").Call("reload")
			} else {
				// either this is the exact same time as we already have, or its the first one and we should remember this instead of the
				// zero time
				serverStartTime = message.ServerStartupTime()
			}
		}

		return websockets.ErrClosed
	}

	// we're going to want increasing delays between reconnect attempts, in case it's real dead to avoid spam
	const initialReconnectDelay = time.Millisecond * 250
	const maxReconnectDelay = time.Second * 15
	reconnectDelay := initialReconnectDelay

	// the main loop, we'll call this once on startup, and again if we need to reconnect
	var loop func()
	loop = func() {
		slog.Debug("attempting to connect auto reload websocket")
		if err := doIt(); err != nil && !errors.Is(err, websockets.ErrClosed) {
			slog.Error("error running auto reload websocket connection", "err", err)
		}

		// connection is dead, so try to reconnect
		slog.Debug("auto reload connection closed, trying again")
		time.Sleep(reconnectDelay)
		reconnectDelay = min(reconnectDelay*2, maxReconnectDelay)
		go loop()
	}

	go loop()
}
