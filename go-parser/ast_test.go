package main

import (
	"errors"
	"io"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestParsing(t *testing.T) {
	for _, test := range []struct {
		input       string
		expected    ASTNode
		expectedErr error
	}{
		{
			"1",
			&NumberNode{Location{0, 0}, 1},
			nil,
		},
		{
			"1 + 2",
			&AddNode{
				&NumberNode{Location{0, 0}, 1},
				&NumberNode{Location{0, 4}, 2},
			},
			nil,
		},
		{
			"1 - 2",
			&SubtractNode{
				&NumberNode{Location{0, 0}, 1},
				&NumberNode{Location{0, 4}, 2},
			},
			nil,
		},
		{
			"1 * 2",
			&MultiplyNode{
				&NumberNode{Location{0, 0}, 1},
				&NumberNode{Location{0, 4}, 2},
			},
			nil,
		},
		{
			"1 / 2",
			&DivideNode{
				&NumberNode{Location{0, 0}, 1},
				&NumberNode{Location{0, 4}, 2},
			},
			nil,
		},
		{
			"(1)",
			&NumberNode{Location{0, 1}, 1},
			nil,
		},
		{
			"-1",
			&NegateNode{
				Location{0, 0},
				&NumberNode{Location{0, 1}, 1},
			},
			nil,
		},
		{
			"(1 + 2) * 3 / 4 + -7",
			&AddNode{
				&DivideNode{
					&MultiplyNode{
						&AddNode{
							&NumberNode{Location{0, 1}, 1},
							&NumberNode{Location{0, 5}, 2},
						},
						&NumberNode{Location{0, 10}, 3},
					},
					&NumberNode{Location{0, 14}, 4},
				},
				&NegateNode{
					Location{0, 18},
					&NumberNode{Location{0, 19}, 7},
				},
			},
			nil,
		},
		{
			"1 /",
			nil,
			errors.Join(
				ErrExpectedToken{T: TokLeftParen},
				ErrExpectedToken{T: TokMinus},
				ErrExpectedToken{T: TokNumber},
				io.EOF,
			),
		},
		{
			"1 / / 5",
			nil,
			errors.Join(
				ErrExpectedToken{T: TokLeftParen},
				ErrExpectedToken{T: TokMinus},
				ErrExpectedToken{T: TokNumber},
				ErrUnhandledToken{Token{Type: TokSlash, Loc: Location{0, 4}, Value: "/"}},
			),
		},
	} {
		input, err := Tokenize([]byte(test.input))
		assert.NoError(t, err, "input = %s", input)
		result, err := Parse(input)
		assert.Equal(t, test.expected, result, "input = %s", input)
		if test.expectedErr == nil {
			assert.NoError(t, err, "input = %s", input)
		} else {
			assert.Equal(t, test.expectedErr, err, "input = %s", input)
		}
	}
}
