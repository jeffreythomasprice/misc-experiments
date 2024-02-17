package main

type LogLevel int

const (
	LvlError LogLevel = iota
	LvlWarning
)

type LogMessage struct {
	Level   LogLevel
	Loc     Location
	Message string
}

type Logger struct {
	Messages []LogMessage
}

func NewLogger() *Logger {
	return &Logger{
		Messages: make([]LogMessage, 0),
	}
}

func (l *Logger) Error(loc Location, message string) {
	l.Messages = append(l.Messages, LogMessage{
		Level:   LvlError,
		Loc:     loc,
		Message: message,
	})
}

func (l *Logger) Warning(loc Location, message string) {
	l.Messages = append(l.Messages, LogMessage{
		Level:   LvlWarning,
		Loc:     loc,
		Message: message,
	})
}
