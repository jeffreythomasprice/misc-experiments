package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEval(t *testing.T) {
	for _, test := range []struct {
		input       string
		expected    int64
		expectedErr error
	}{
		{
			"1",
			1,
			nil,
		},
		{
			"1 + 2",
			3,
			nil,
		},
		{
			"1 - 2",
			-1,
			nil,
		},
		{
			"1 * 2",
			2,
			nil,
		},
		{
			"1 / 2",
			0,
			nil,
		},
		{
			"(1 + 2) * 3 + -4",
			5,
			nil,
		},
	} {
		input, err := Tokenize([]byte(test.input))
		assert.NoError(t, err, "input = %s", input)
		ast, err := Parse(input)
		assert.NoError(t, err, "input = %s", input)
		result, err := Eval(ast)
		assert.Equal(t, test.expected, result)
		if test.expectedErr == nil {
			assert.NoError(t, err, "input = %s", input)
		} else {
			assert.Equal(t, test.expectedErr, err, "input = %s", input)
		}
	}
}
