package websockets

import "time"

type ReconnectStrategy interface {
	Reset()
	Next() (time.Duration, bool)
}

type constantReconnectStrategy struct {
	d time.Duration
}

var _ ReconnectStrategy = (*constantReconnectStrategy)(nil)

func Every(d time.Duration) ReconnectStrategy {
	return &constantReconnectStrategy{d: d}
}

// Reset implements ReconnectStrategy.
func (*constantReconnectStrategy) Reset() {}

// Next implements ReconnectStrategy.
func (strat *constantReconnectStrategy) Next() (time.Duration, bool) {
	return strat.d, true
}
