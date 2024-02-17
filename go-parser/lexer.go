package main

import (
	"bytes"
	"fmt"
	"regexp"
	"unicode/utf8"
)

type TokenType int

const (
	TokNumber TokenType = iota
	TokPlus
	TokMinus
	TokAsterisk
	TokSlash
	TokLeftParen
	TokRightParen
)

func (t TokenType) String() string {
	switch t {
	case TokNumber:
		return "NUMBER"
	case TokPlus:
		return "PLUS"
	case TokMinus:
		return "MINUS"
	case TokAsterisk:
		return "ASTERISK"
	case TokSlash:
		return "SLASH"
	case TokLeftParen:
		return "LEFT_PAREN"
	case TokRightParen:
		return "RIGHT_PAREN"
	default:
		panic(fmt.Sprintf("unhandled enum: %d", t))
	}
}

type Token struct {
	Type  TokenType
	Loc   Location
	Value string
}

func (t Token) String() string {
	return fmt.Sprintf("Token(%v, %v, %s)", t.Type, t.Loc, t.Value)
}

type ErrUnhandledRune struct {
	R rune
}

var _ error = ErrUnhandledRune{}

// Error implements error.
func (err ErrUnhandledRune) Error() string {
	return fmt.Sprintf("unhandled rune: '%c'", err.R)
}

type ErrUnhandledByte struct {
	B byte
}

var _ error = ErrUnhandledByte{}

// Error implements error.
func (err ErrUnhandledByte) Error() string {
	return fmt.Sprintf("unhandled rune: %2x", err.B)
}

var numberRegexp *regexp.Regexp
var whitespaceRegexp *regexp.Regexp

func init() {
	numberRegexp = regexp.MustCompile("^[0-9]+")
	whitespaceRegexp = regexp.MustCompile(`^\s+`)
}

func Tokenize(input []byte) ([]Token, error) {
	results := make([]Token, 0)
	loc := Location{Line: 0, Column: 0}

	matchRegexp := func(r *regexp.Regexp, t TokenType) bool {
		indices := r.FindIndex(input)
		if indices == nil {
			return false
		}
		if indices[0] > 0 {
			return false
		}
		result := Token{
			Type:  t,
			Loc:   loc,
			Value: string(input[indices[0]:indices[1]]),
		}
		results = append(results, result)
		loc.AdvanceString(result.Value)
		input = input[indices[1]:]
		return true
	}

	matchLiteral := func(l string, t TokenType) bool {
		if !bytes.HasPrefix(input, []byte(l)) {
			return false
		}
		results = append(results, Token{
			Type:  t,
			Loc:   loc,
			Value: l,
		})
		loc.AdvanceString(l)
		input = input[len(l):]
		return true
	}

	skipRegexp := func(r *regexp.Regexp) {
		indices := r.FindIndex(input)
		if indices == nil {
			return
		}
		if indices[0] > 0 {
			return
		}
		loc.AdvanceBytes(input[indices[0]:indices[1]])
		input = input[indices[1]:]
	}

	for len(input) > 0 {
		skipRegexp(whitespaceRegexp)

		if matchRegexp(numberRegexp, TokNumber) {
			continue
		}
		if matchLiteral("+", TokPlus) {
			continue
		}
		if matchLiteral("-", TokMinus) {
			continue
		}
		if matchLiteral("*", TokAsterisk) {
			continue
		}
		if matchLiteral("/", TokSlash) {
			continue
		}
		if matchLiteral("(", TokLeftParen) {
			continue
		}
		if matchLiteral(")", TokRightParen) {
			continue
		}

		skipRegexp(whitespaceRegexp)
		if len(input) == 0 {
			continue
		}

		r, _ := utf8.DecodeRune(input)
		if r == utf8.RuneError {
			return results, ErrUnhandledByte{B: input[0]}
		} else {
			return results, ErrUnhandledRune{R: r}
		}
	}
	return results, nil
}
