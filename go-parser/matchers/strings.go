package matchers

import "fmt"

type Position struct {
	Line, Column int
}

func (pos Position) String() string {
	return fmt.Sprintf("[%v:%v]", pos.Line, pos.Column)
}

func (pos Position) Advance(r rune) Position {
	if r == '\n' {
		return Position{
			Line:   pos.Line + 1,
			Column: 0,
		}
	}
	return Position{
		Line:   pos.Line,
		Column: pos.Column + 1,
	}
}

type PosStr struct {
	Pos Position
	S   string
}
