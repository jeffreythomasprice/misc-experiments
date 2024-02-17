package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestLocationAdvanceRune(t *testing.T) {
	l := Location{Line: 0, Column: 0}

	l.AdvanceRune('a')
	assert.Equal(t, Location{Line: 0, Column: 1}, l)

	l.AdvanceRune('b')
	assert.Equal(t, Location{Line: 0, Column: 2}, l)

	l.AdvanceRune('c')
	assert.Equal(t, Location{Line: 0, Column: 3}, l)

	l.AdvanceRune('\n')
	assert.Equal(t, Location{Line: 1, Column: 0}, l)

	l.AdvanceRune('d')
	assert.Equal(t, Location{Line: 1, Column: 1}, l)
}

func TestLocationAdvanceString(t *testing.T) {
	l := Location{Line: 0, Column: 0}
	l.AdvanceString("abc\n456")
	assert.Equal(t, Location{Line: 1, Column: 3}, l)
}

func TestLocationAdvanceBytes(t *testing.T) {
	l := Location{Line: 0, Column: 0}
	l.AdvanceBytes([]byte("abc\n456"))
	assert.Equal(t, Location{Line: 1, Column: 3}, l)
}

func TestLocationString(t *testing.T) {
	l := Location{Line: 1, Column: 2}
	assert.Equal(t, "[2:3]", l.String())
}
