package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestLogger(t *testing.T) {
	l := NewLogger()
	assert.Equal(t, []LogMessage{}, l.Messages)
	l.Warning(Location{Line: 1, Column: 2}, "foo")
	l.Error(Location{Line: 3, Column: 4}, "bar")
	l.Error(Location{Line: 5, Column: 6}, "baz")
	assert.Equal(t, []LogMessage{
		{
			Level:   LvlWarning,
			Loc:     Location{Line: 1, Column: 2},
			Message: "foo",
		},
		{
			Level:   LvlError,
			Loc:     Location{Line: 3, Column: 4},
			Message: "bar",
		},
		{
			Level:   LvlError,
			Loc:     Location{Line: 5, Column: 6},
			Message: "baz",
		},
	}, l.Messages)
}
