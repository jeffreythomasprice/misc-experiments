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

type backoffReconnectStrategy struct {
	initial, max, current time.Duration
}

var _ ReconnectStrategy = (*backoffReconnectStrategy)(nil)

func Backoff(initial, max time.Duration) ReconnectStrategy {
	return &backoffReconnectStrategy{
		initial: initial,
		max:     max,
		current: initial,
	}
}

// Reset implements ReconnectStrategy.
func (strat *backoffReconnectStrategy) Reset() {
	strat.current = strat.initial
}

// Next implements ReconnectStrategy.
func (strat *backoffReconnectStrategy) Next() (time.Duration, bool) {
	result := strat.current
	strat.current = min(strat.current*2, strat.max)
	return result, true
}
