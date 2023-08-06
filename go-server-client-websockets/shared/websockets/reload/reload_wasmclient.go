//go:build js && wasm

package reload

import (
	"log/slog"
	"shared"
	"shared/websockets"
	"time"
)

func StartAutoReloadClient(addr string) error {
	connection, err := websockets.NewWebsocketConnection(addr)
	if err != nil {
		return err
	}

	jsonConnection := websockets.NewJsonWebsocketConnection[ServerHelloMessage](connection)

	// always set to now
	// if the client restarts it'll call this function again, getting us a new time
	clientStartTime := time.Now()

	// will be set to the time the server reports when we connect
	// until then it's the zero time
	var serverStartTime time.Time

	go func() {
		for message := range jsonConnection.Incoming() {
			// TODO should be trace
			slog.Debug(
				"incoming message",
				"time", message.ServerStartupTime().Format(shared.ISO8601_MILLIS_FORMAT),
			)

			if serverStartTime.IsZero() && time.Now().Sub(clientStartTime) > time.Second*10 {
				slog.Info("first server hello received after client has already been up for a while, restarting")
				// TODO JEFF should restart here
			} else if !message.ServerStartupTime().Equal(serverStartTime) {
				slog.Info("server hello time doesn't match, restarting")
				// TODO JEFF should restart here
			} else {
				// either this is the exact same time as we already have, or its the first one and we should remember this instead of the
				// zero time
				serverStartTime = message.ServerStartupTime()
			}
		}

		slog.Debug("auto reload connection closed, trying again")
		// TODO JEFF actually reconnect, exponential backoff?
	}()

	return nil
}
