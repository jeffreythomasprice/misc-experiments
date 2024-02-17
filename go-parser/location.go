package main

import "fmt"

type Location struct {
	Line, Column int
}

func (l Location) String() string {
	return fmt.Sprintf("[%d:%d]", l.Line+1, l.Column+1)
}

func (l *Location) AdvanceRune(r rune) {
	if r == '\n' {
		l.Line++
		l.Column = 0
	} else {
		l.Column++
	}
}

func (l *Location) AdvanceString(s string) {
	for _, r := range s {
		l.AdvanceRune(r)
	}
}

func (l *Location) AdvanceBytes(b []byte) {
	l.AdvanceString(string(b))
}
