import calculator
import gleam/list
import gleam/option

pub fn calculator_test() {
  [
    #("1", option.Some(1.0)),
    #("  1    ", option.Some(1.0)),
    #("42.5", option.Some(42.5)),
    #("1 + 2 * 3", option.Some(7.0)),
    #("-  (1 + 2) * 3", option.Some(-9.0)),
    #("13/2.5", option.Some(5.2)),
  ]
  |> list.each(fn(x) {
    let #(input, expected) = x
    let actual = calculator.evaluate(input)
    assert actual == expected as input
  })
}
