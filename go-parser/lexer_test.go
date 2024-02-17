package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestTokenize(t *testing.T) {
	tokens, err := Tokenize([]byte(" 123 + * -\n\t\t / ( ) \t456 "))
	assert.Equal(t, []Token{
		{
			Type:  TokNumber,
			Loc:   Location{Line: 0, Column: 1},
			Value: "123",
		},
		{
			Type:  TokPlus,
			Loc:   Location{Line: 0, Column: 5},
			Value: "+",
		},
		{
			Type:  TokAsterisk,
			Loc:   Location{Line: 0, Column: 7},
			Value: "*",
		},
		{
			Type:  TokMinus,
			Loc:   Location{Line: 0, Column: 9},
			Value: "-",
		},
		{
			Type:  TokSlash,
			Loc:   Location{Line: 1, Column: 3},
			Value: "/",
		},
		{
			Type:  TokLeftParen,
			Loc:   Location{Line: 1, Column: 5},
			Value: "(",
		},
		{
			Type:  TokRightParen,
			Loc:   Location{Line: 1, Column: 7},
			Value: ")",
		},
		{
			Type:  TokNumber,
			Loc:   Location{Line: 1, Column: 10},
			Value: "456",
		},
	}, tokens)
	assert.NoError(t, err)
}
