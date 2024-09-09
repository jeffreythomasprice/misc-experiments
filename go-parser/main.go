package main

import (
	. "experiment/matchers"
	"fmt"
	"math"
	"strconv"
)

var matchUint32 =
/*
0
[1-9][0-9]*
*/
Any2(
	Map[rune, uint32](
		SpecificRune('0'),
		func(pos Position, value rune) (uint32, error) {
			return 0, nil
		},
	),
	Map[Match2Result[rune, []rune], uint32](
		Match2(
			RuneRange('1', '9'),
			Range(
				RuneRange('0', '9'),
				0,
				10,
			),
		),
		func(pos Position, value Match2Result[rune, []rune]) (uint32, error) {
			s := string(append([]rune{value.Result1}, value.Result2...))
			i, err := strconv.ParseInt(s, 10, 64)
			if err != nil {
				return 0, err
			}
			if i < 0 || i > math.MaxUint32 {
				return 0, fmt.Errorf("%v outside valid range for uint32", s)
			}
			return uint32(i), nil
		},
	),
)

func main() {
	result, remainder, err := matchUint32(PosStr{S: "0123"})
	fmt.Printf("result = %v\n", result)
	fmt.Printf("remainder = %v\n", remainder)
	fmt.Printf("err = %v\n", err)
}
