package shared

import (
	"log/slog"
	"os"
)

func InitSlog() {
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stdout,
		&slog.HandlerOptions{
			Level: slog.LevelDebug,
		},
	)))
}
