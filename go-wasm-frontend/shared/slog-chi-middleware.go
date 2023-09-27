package shared

import (
	"log/slog"
	"net/http"
	"time"

	"github.com/go-chi/chi/middleware"
)

type SlogLogFormatter struct{}

var _ middleware.LogFormatter = (*SlogLogFormatter)(nil)

type SlogLogEntry struct {
	request *http.Request
}

var _ middleware.LogEntry = (*SlogLogEntry)(nil)

// NewLogEntry implements middleware.LogFormatter.
func (*SlogLogFormatter) NewLogEntry(r *http.Request) middleware.LogEntry {
	return &SlogLogEntry{request: r}
}

// Panic implements middleware.LogEntry.
func (*SlogLogEntry) Panic(v interface{}, stack []byte) {
	slog.Error("chi handler panic", "v", v, "stack", string(stack))
}

// Write implements middleware.LogEntry.
func (entry *SlogLogEntry) Write(status int, bytes int, header http.Header, elapsed time.Duration, extra interface{}) {
	slog.Debug(
		"chi handler",
		"method", entry.request.Method,
		"url", entry.request.URL.String(),
		"remoteAddr", entry.request.RemoteAddr,
		"status", status,
		"bytes", bytes,
		"elapsed", elapsed,
	)
}
