package matchers

import (
	"fmt"
	"io"
	"unicode/utf8"
)

type ExpectedMatch struct {
	Pos              Position
	Actual, Expected string
}

var _ error = ExpectedMatch{}

// Error implements error.
func (e ExpectedMatch) Error() string {
	return fmt.Sprintf("Expected(pos=%v, expected=%v, actual=%v)", e.Pos, e.Expected, e.Actual)
}

type Matcher[T any] func(input PosStr) (result T, remainder PosStr, err error)

func Map[T, R any](m Matcher[T], f func(pos Position, value T) (R, error)) Matcher[R] {
	return func(input PosStr) (result R, remainder PosStr, err error) {
		var temp T
		temp, remainder, err = m(input)
		if err != nil {
			return
		}
		result, err = f(input.Pos, temp)
		return
	}
}

type Match2Result[T1, T2 any] struct {
	Result1 T1
	Result2 T2
}

func Match2[T1, T2 any](m1 Matcher[T1], m2 Matcher[T2]) Matcher[Match2Result[T1, T2]] {
	return func(input PosStr) (result Match2Result[T1, T2], remainder PosStr, err error) {
		result.Result1, remainder, err = m1(input)
		if err != nil {
			return
		}
		result.Result2, remainder, err = m2(remainder)
		return
	}
}

// TODO match3, match4, ...

func Any2[T any](m1 Matcher[T], m2 Matcher[T]) Matcher[T] {
	return func(input PosStr) (result T, remainder PosStr, err error) {
		result, remainder, err = m1(input)
		if err == nil {
			return
		}
		err1 := err
		result, remainder, err = m2(input)
		if err == nil {
			return
		}
		err2 := err
		err = fmt.Errorf("no match, checked [%v, %v]", err1, err2)
		return
	}
}

// TODO any3, any4, ...

type RepeatOptions struct {
	min *uint
	max *uint
}

func Repeat[T any](m Matcher[T], options RepeatOptions) Matcher[[]T] {
	isGood := func(count uint) bool {
		if options.min != nil && count < *options.min {
			return false
		}
		if options.max != nil && count > *options.max {
			return false
		}
		return true
	}
	return func(input PosStr) (result []T, remainder PosStr, err error) {
		// loop over the whole input until an exit condition occurs
		remainder = input
		for {
			// we're done if we have a full list and matching again would put us over the limit
			if isGood(uint(len(result))) && !isGood(uint(len(result)+1)) {
				return
			}
			// match the next one
			var next T
			next, remainder, err = m(remainder)
			// no match
			if err != nil {
				// but we had enough already, so let this be the next matcher's problem and don't say we had an error
				if isGood(uint(len(result))) {
					err = nil
				}
				// either way, we're done
				return
			}
			// successful match, add it to the list and start over
			result = append(result, next)
		}
	}
}

func AtLeast[T any](m Matcher[T], count uint) Matcher[[]T] {
	var options RepeatOptions
	options.min = &count
	return Repeat[T](m, options)
}

func AtMost[T any](m Matcher[T], count uint) Matcher[[]T] {
	var options RepeatOptions
	options.max = &count
	return Repeat[T](m, options)
}

func Range[T any](m Matcher[T], min, max uint) Matcher[[]T] {
	if min > max {
		min, max = max, min
	}
	var options RepeatOptions
	options.min = &min
	options.max = &max
	return Repeat[T](m, options)
}

type OptionalResult[T any] struct {
	Matched bool
	Value   T
}

func Optional[T any](m Matcher[T]) Matcher[OptionalResult[T]] {
	return Map[[]T, OptionalResult[T]](
		Range[T](m, 0, 1),
		func(pos Position, value []T) (OptionalResult[T], error) {
			var result OptionalResult[T]
			if len(value) == 0 {
				result.Matched = false
			} else {
				result.Matched = true
				result.Value = value[0]
			}
			return result, nil
		},
	)
}

func AnyRune() Matcher[rune] {
	return func(input PosStr) (result rune, remainder PosStr, err error) {
		for pos, result := range input.S {
			start_of_next := pos + utf8.RuneLen(result)
			remainder = PosStr{Pos: input.Pos.Advance(result), S: input.S[start_of_next:]}
			return result, remainder, nil
		}
		remainder = input
		err = io.EOF
		return
	}
}

func SpecificRune(r rune) Matcher[rune] {
	return Map(AnyRune(), func(pos Position, actual rune) (rune, error) {
		if actual != r {
			return 0, ExpectedMatch{
				Pos:      pos,
				Actual:   string(actual),
				Expected: string(r),
			}
		}
		return actual, nil
	})
}

func RuneRange(r1, r2 rune) Matcher[rune] {
	if r1 > r2 {
		r1, r2 = r2, r1
	}
	return Map(AnyRune(), func(pos Position, actual rune) (rune, error) {
		if actual < r1 || actual > r2 {
			return 0, ExpectedMatch{
				Pos:      pos,
				Actual:   string(actual),
				Expected: fmt.Sprintf("[%v,%v]", r1, r2),
			}
		}
		return actual, nil
	})
}
